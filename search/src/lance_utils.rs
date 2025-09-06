use std::path::PathBuf;
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
        
        // Perform vector search using LanceDB's built-in vector search
        let mut results = table
            .vector_search(query_embedding)?
            .distance_type(DistanceType::Cosine)
            .limit(limit * 10) // Get 10x more for reranking
            .execute()
            .await?;
        
        // Convert results to our format
        let mut search_results = Vec::new();
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
                
                // Try different possible distance column names
                let score = if let Some(distance_col) = batch.column_by_name("_distance") {
                    let distance = distance_col.as_any().downcast_ref::<arrow_array::Float32Array>().unwrap().value(i);
                    1.0 - distance
                } else if let Some(distance_col) = batch.column_by_name("distance") {
                    let distance = distance_col.as_any().downcast_ref::<arrow_array::Float32Array>().unwrap().value(i);
                    1.0 - distance
                } else if let Some(score_col) = batch.column_by_name("_score") {
                    score_col.as_any().downcast_ref::<arrow_array::Float32Array>().unwrap().value(i)
                } else {
                    println!("No distance/score column found, using 0.5");
                    0.5 // Fallback score
                };
                
                search_results.push(LanceSearchResult {
                    score,
                    id,
                    category,
                    path,
                    content,
                });
            }
        }

        // No reranking - using raw vector similarity scores

        // Sort by score (highest first)
        search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top N results
        search_results.truncate(limit);

        Ok(search_results)
    }
    

    pub async fn get_stats(&self) -> Result<String, anyhow::Error> {
        let table = self.db.open_table(&self.table_name).execute().await?;
        let count = table.count_rows(None).await?;
        Ok(format!("LanceDB table '{}' with {} documents", self.table_name, count))
    }
}
