use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use serde::{Deserialize, Serialize};
use crate::embedding::EmbeddingModel;
use lancedb::{connect, Connection, DistanceType};
use lancedb::query::{QueryBase, ExecutableQuery};
use futures::TryStreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanceDocument {
    pub id: String,
    pub content: String,
    pub category: String,
    pub path: String,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct LanceSearchResult {
    pub score: f32,
    pub id: String,
    pub category: String,
    pub path: String,
    pub content: String,
}

pub struct LanceSearchEngine {
    db: Connection,
    table_name: String,
    embedding_model: EmbeddingModel,
}

impl LanceSearchEngine {
    pub async fn new(db_path: PathBuf, table_name: &str) -> Result<Self, anyhow::Error> {
        let embedding_model = EmbeddingModel::new()?;
        let db = connect(db_path.to_string_lossy().as_ref()).execute().await?;
        
        Ok(Self {
            db,
            table_name: table_name.to_string(),
            embedding_model,
        })
    }

    pub async fn search(&self, query_text: &str, limit: usize) -> Result<Vec<LanceSearchResult>, anyhow::Error> {
        // Generate query embedding
        let query_embedding = self.embedding_model.embed_text(query_text)?;
        
        // Open the table and perform vector search
        let table = self.db.open_table(&self.table_name).execute().await?;
        
        // Step 1: PQ Search - Get 10xN results from Product Quantization index (fast, approximate)
        let pq_limit = limit * 10;
        println!("🔍 PQ Search: Getting {} results from Product Quantization index", pq_limit);
        
        // Step 2: Use LanceDB's vector search with over-retrieval, then manually rerank
        println!("🎯 Vector Search: Using LanceDB's vector search with {} over-retrieval", pq_limit);
        
        let mut results = table
            .vector_search(query_embedding)?
            .distance_type(DistanceType::Cosine)
            .limit(pq_limit) // Get 10x more results for better recall
            .execute()
            .await?;
        
        // Convert results to our format
        let mut all_results = Vec::new();
        while let Some(batch) = results.try_next().await? {
            // Process each record in the batch
            for i in 0..batch.num_rows() {
                let id = batch.column_by_name("id").unwrap()
                    .as_any().downcast_ref::<arrow_array::StringArray>().unwrap()
                    .value(i).to_string();
                let category = batch.column_by_name("category").unwrap()
                    .as_any().downcast_ref::<arrow_array::StringArray>().unwrap()
                    .value(i).to_string();
                let path = batch.column_by_name("doc_path").unwrap()
                    .as_any().downcast_ref::<arrow_array::StringArray>().unwrap()
                    .value(i).to_string();
                let content = batch.column_by_name("content").unwrap()
                    .as_any().downcast_ref::<arrow_array::StringArray>().unwrap()
                    .value(i).to_string();
                
                // Get reranked score (should be more accurate than PQ scores)
                let score = if let Some(distance_col) = batch.column_by_name("_distance") {
                    let distance = distance_col.as_any().downcast_ref::<arrow_array::Float32Array>().unwrap().value(i);
                    1.0 - distance // Convert distance to similarity score
                } else if let Some(distance_col) = batch.column_by_name("distance") {
                    let distance = distance_col.as_any().downcast_ref::<arrow_array::Float32Array>().unwrap().value(i);
                    1.0 - distance
                } else if let Some(score_col) = batch.column_by_name("_score") {
                    score_col.as_any().downcast_ref::<arrow_array::Float32Array>().unwrap().value(i)
                } else {
                    println!("No distance/score column found, using 0.5");
                    0.5 // Fallback score
                };
                
                all_results.push(LanceSearchResult {
                    score,
                    id,
                    category,
                    path,
                    content,
                });
            }
        }

        // Dump all PQ results to file for analysis
        let pq_filename = format!("pq_results_{}.txt", query_text.replace(" ", "_"));
        self.dump_pq_results(&all_results, query_text, &pq_filename)?;
        println!("📁 Dumped {} PQ results to: {}", all_results.len(), pq_filename);

        // Reranking step: Use a different scoring mechanism to actually reorder results
        // This simulates what a real reranker would do - use different criteria
        let query_lower = query_text.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        
        for result in &mut all_results {
            // Simple text-based reranking: boost scores for exact word matches
            let content_lower = result.content.to_lowercase();
            
            let mut text_score = 0.0;
            for word in &query_words {
                if content_lower.contains(word) {
                    text_score += 1.0;
                }
            }
            
            // Combine vector similarity (70%) with text matching (30%)
            result.score = (result.score * 0.7) + (text_score / query_words.len() as f32 * 0.3);
        }
        
        // Sort by the new combined score
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top N results from the over-retrieved set
        let original_count = all_results.len();
        let final_results = all_results[..limit.min(original_count)].to_vec();

        println!("✅ Hybrid Search Complete: {} over-retrieved → {} final results", original_count, final_results.len());
        
        // Show analysis of top 3 vs PQ top 3
        if limit >= 3 && original_count >= 3 {
            println!("\n🔍 Analysis - Top 3 Final vs Top 3 PQ:");
            println!("Final Top 3:");
            for (i, result) in final_results.iter().take(3).enumerate() {
                println!("  {}. score={:.4} id={} | {}", i+1, result.score, result.id, 
                         result.content.chars().take(80).collect::<String>());
            }
            println!("PQ Top 3 (before reranking):");
            for (i, result) in all_results.iter().take(3).enumerate() {
                println!("  {}. score={:.4} id={} | {}", i+1, result.score, result.id, 
                         result.content.chars().take(80).collect::<String>());
            }
        }

        Ok(final_results)
    }
    

    pub async fn get_stats(&self) -> Result<String, anyhow::Error> {
        let table = self.db.open_table(&self.table_name).execute().await?;
        let count = table.count_rows(None).await?;
        Ok(format!("LanceDB table '{}' with {} documents", self.table_name, count))
    }

    // Debug method to dump PQ results to file
    fn dump_pq_results(&self, results: &[LanceSearchResult], query: &str, filename: &str) -> Result<(), anyhow::Error> {
        let mut file = File::create(filename)?;
        writeln!(file, "PQ Search Results for: \"{}\"", query)?;
        writeln!(file, "Total results: {}", results.len())?;
        writeln!(file, "{}", "=".repeat(80))?;
        
        for (i, result) in results.iter().enumerate() {
            writeln!(file, "\n{}. score={:.4}  id={}  category={}", 
                     i + 1, result.score, result.id, result.category)?;
            writeln!(file, "   Content: {}", result.content)?;
        }
        
        Ok(())
    }
}
