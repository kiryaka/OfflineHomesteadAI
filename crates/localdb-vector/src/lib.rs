use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use lancedb::{connect, Connection};
use lancedb::query::{QueryBase, ExecutableQuery};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use arrow_array::{RecordBatch, RecordBatchIterator, Int32Array, FixedSizeListArray, StringArray};
use arrow_schema::{Schema, Field, DataType};

use localdb_core::data_processor::DocumentChunk;
use localdb_embed::{Embedder, get_default_embedder};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanceDocument {
    pub id: String,
    pub doc_id: String,
    pub doc_path: String,
    pub category: String,
    pub category_text: String,
    pub content: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub vector: Vec<f32>,
}

pub struct LanceDbIndexer { db: Connection, table_name: String, embedder: Box<dyn Embedder> }

impl LanceDbIndexer {
    pub async fn new(db_path: &Path, table_name: &str) -> Result<Self> {
        let embedder = get_default_embedder()?;
        let db = connect(db_path.to_string_lossy().as_ref()).execute().await?;
        Ok(Self { db, table_name: table_name.to_string(), embedder })
    }

    pub async fn index_chunks(&self, chunks: &[DocumentChunk]) -> Result<()> {
        if chunks.is_empty() { println!("No chunks to index"); return Ok(()); }
        println!("Indexing {} chunks into LanceDB table: {}", chunks.len(), self.table_name);
        let pb = ProgressBar::new(chunks.len() as u64);
        pb.set_style(ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} chunks ({percent}%) {msg}").unwrap().progress_chars("#>-") );
        let mut processed = 0usize; let mut batch_docs = Vec::new(); let batch_size = 1000usize;
        for (i, chunk) in chunks.iter().enumerate() {
            let embedding = self.embedder.embed_text(&chunk.content)?;
            let doc = LanceDocument { id: chunk.id.clone(), doc_id: chunk.doc_id.clone(), doc_path: chunk.doc_path.clone(), category: chunk.category.clone(), category_text: chunk.category_text.clone(), content: chunk.content.clone(), chunk_index: chunk.chunk_index, total_chunks: chunk.total_chunks, vector: embedding };
            batch_docs.push(doc); processed += 1; pb.set_position(processed as u64); pb.set_message(format!("Processing chunk {}", i + 1));
            if batch_docs.len() >= batch_size || i == chunks.len() - 1 { self.insert_batch(&batch_docs).await?; batch_docs.clear(); if processed % 1000 == 0 { println!("\n📦 Processed batch of 1000 chunks..."); } }
        }
        pb.finish_with_message("✅ LanceDB indexing completed!");
        println!("📊 Successfully indexed {} chunks into LanceDB", processed);
        Ok(())
    }

    async fn insert_batch(&self, docs: &[LanceDocument]) -> Result<()> {
        if docs.is_empty() { return Ok(()); }
        let record_batch = self.docs_to_record_batch(docs)?; let schema = record_batch.schema();
        let reader = Box::new(RecordBatchIterator::new(vec![Ok(record_batch)].into_iter(), schema));
        if self.db.table_names().execute().await?.contains(&self.table_name) {
            self.db.open_table(&self.table_name).execute().await?.add(reader).execute().await?;
        } else {
            self.db.create_table(&self.table_name, reader).execute().await?;
        }
        Ok(())
    }

    fn docs_to_record_batch(&self, docs: &[LanceDocument]) -> Result<RecordBatch> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("doc_id", DataType::Utf8, false),
            Field::new("doc_path", DataType::Utf8, false),
            Field::new("category", DataType::Utf8, false),
            Field::new("category_text", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("chunk_index", DataType::Int32, false),
            Field::new("total_chunks", DataType::Int32, false),
            Field::new("vector", DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 1024), true),
        ]));
        let mut ids = Vec::new(); let mut doc_ids = Vec::new(); let mut doc_paths = Vec::new(); let mut categories = Vec::new(); let mut category_texts = Vec::new(); let mut contents = Vec::new(); let mut chunk_indices = Vec::new(); let mut total_chunks = Vec::new(); let mut vectors: Vec<Option<Vec<Option<f32>>>> = Vec::new();
        for doc in docs { ids.push(doc.id.clone()); doc_ids.push(doc.doc_id.clone()); doc_paths.push(doc.doc_path.clone()); categories.push(doc.category.clone()); category_texts.push(doc.category_text.clone()); contents.push(doc.content.clone()); chunk_indices.push(doc.chunk_index as i32); total_chunks.push(doc.total_chunks as i32); vectors.push(Some(doc.vector.iter().map(|&x| Some(x)).collect())); }
        let record_batch = RecordBatch::try_new(schema, vec![
            Arc::new(StringArray::from(ids)),
            Arc::new(StringArray::from(doc_ids)),
            Arc::new(StringArray::from(doc_paths)),
            Arc::new(StringArray::from(categories)),
            Arc::new(StringArray::from(category_texts)),
            Arc::new(StringArray::from(contents)),
            Arc::new(Int32Array::from(chunk_indices)),
            Arc::new(Int32Array::from(total_chunks)),
            Arc::new(FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(vectors.into_iter(), 1024)),
        ])?;
        Ok(record_batch)
    }
}

pub struct LanceSearchEngine { db: Connection, table_name: String, embedder: Box<dyn Embedder> }

impl LanceSearchEngine {
    pub async fn new(db_path: std::path::PathBuf, table_name: &str) -> Result<Self, anyhow::Error> {
        let embedder = get_default_embedder()?; let db = connect(db_path.to_string_lossy().as_ref()).execute().await?;
        Ok(Self { db, table_name: table_name.to_string(), embedder })
    }

    pub async fn search(&self, query_text: &str, limit: usize) -> Result<Vec<LanceSearchResult>, anyhow::Error> {
        let query_embedding = self.embedder.embed_text(query_text)?; let table = self.db.open_table(&self.table_name).execute().await?;
        let pq_limit = limit * 10; let mut results = table.vector_search(query_embedding)?.limit(pq_limit).execute().await?;
        let mut all_results = Vec::new();
        while let Some(batch) = futures::TryStreamExt::try_next(&mut results).await? {
            for i in 0..batch.num_rows() {
                let id = batch.column_by_name("id").unwrap().as_any().downcast_ref::<arrow_array::StringArray>().unwrap().value(i).to_string();
                let category = batch.column_by_name("category").unwrap().as_any().downcast_ref::<arrow_array::StringArray>().unwrap().value(i).to_string();
                let path = batch.column_by_name("doc_path").unwrap().as_any().downcast_ref::<arrow_array::StringArray>().unwrap().value(i).to_string();
                let content = batch.column_by_name("content").unwrap().as_any().downcast_ref::<arrow_array::StringArray>().unwrap().value(i).to_string();
                let score = if let Some(distance_col) = batch.column_by_name("_distance") { 1.0 - distance_col.as_any().downcast_ref::<arrow_array::Float32Array>().unwrap().value(i) }
                            else if let Some(distance_col) = batch.column_by_name("distance") { 1.0 - distance_col.as_any().downcast_ref::<arrow_array::Float32Array>().unwrap().value(i) }
                            else if let Some(score_col) = batch.column_by_name("_score") { score_col.as_any().downcast_ref::<arrow_array::Float32Array>().unwrap().value(i) }
                            else { 0.5 };
                all_results.push(LanceSearchResult { score, id, category, path, content });
            }
        }
        // Simple rerank
        let query_lower = query_text.to_lowercase(); let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        for result in &mut all_results { let content_lower = result.content.to_lowercase(); let mut text_score = 0.0; for word in &query_words { if content_lower.contains(word) { text_score += 1.0; } } result.score = (result.score * 0.7) + (text_score / query_words.len() as f32 * 0.3); }
        all_results.sort_by(|a,b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(all_results.into_iter().take(limit).collect())
    }
}

#[derive(Debug, Clone)]
pub struct LanceSearchResult { pub score: f32, pub id: String, pub category: String, pub path: String, pub content: String }
