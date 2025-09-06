use crate::data_processor::DocumentChunk;
use crate::embedding::EmbeddingModel;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle};
use lancedb::{connect, Connection};
use arrow_array::{RecordBatch, RecordBatchIterator, Int32Array, FixedSizeListArray, StringArray};
use arrow_schema::{Schema, Field, DataType};
use std::sync::Arc;

/// LanceDB document for indexing
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
    pub vector: Vec<f32>, // 1024 dimensions for BGE-large-en
}

/// LanceDB indexer for vector search
/// Note: This is a placeholder implementation.
/// In production, you would integrate with the actual LanceDB API.
pub struct LanceDbIndexer {
    db: Connection,
    table_name: String,
    embedding_model: EmbeddingModel,
}

impl LanceDbIndexer {
    /// Create a new LanceDB indexer
    pub async fn new(db_path: &Path, table_name: &str) -> Result<Self> {
        let embedding_model = EmbeddingModel::new()?;
        
        // Connect to LanceDB
        let db = connect(db_path.to_string_lossy().as_ref()).execute().await?;
        
        Ok(Self {
            db,
            table_name: table_name.to_string(),
            embedding_model,
        })
    }

    /// Index document chunks into LanceDB
    pub async fn index_chunks(&self, chunks: &[DocumentChunk]) -> Result<()> {
        if chunks.is_empty() {
            println!("No chunks to index");
            return Ok(());
        }

        println!(
            "Indexing {} chunks into LanceDB table: {}",
            chunks.len(),
            self.table_name
        );

        // Create progress bar
        let pb = ProgressBar::new(chunks.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} chunks ({percent}%) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Process chunks with progress tracking and real LanceDB storage
        let mut processed = 0;
        let mut batch_docs = Vec::new();
        let batch_size = 1000;

        for (i, chunk) in chunks.iter().enumerate() {
            // Generate embedding for the content using BGE-large-en
            let embedding = self.embedding_model.embed_text(&chunk.content)?;

            // Create LanceDB document
            let doc = LanceDocument {
                id: chunk.id.clone(),
                doc_id: chunk.doc_id.clone(),
                doc_path: chunk.doc_path.clone(),
                category: chunk.category.clone(),
                category_text: chunk.category_text.clone(),
                content: chunk.content.clone(),
                chunk_index: chunk.chunk_index,
                total_chunks: chunk.total_chunks,
                vector: embedding,
            };

            batch_docs.push(doc);
            processed += 1;
            pb.set_position(processed as u64);
            pb.set_message(format!("Processing chunk {}", i + 1));

            // Insert batch when we reach batch size or end of chunks
            if batch_docs.len() >= batch_size || i == chunks.len() - 1 {
                self.insert_batch(&batch_docs).await?;
                batch_docs.clear();
                
                if processed % 1000 == 0 {
                    println!("\n📦 Processed batch of 1000 chunks...");
                }
            }
        }

        pb.finish_with_message("✅ LanceDB indexing completed!");
        println!("📊 Successfully indexed {} chunks into LanceDB", processed);

        Ok(())
    }

    /// Insert a batch of documents into LanceDB
    async fn insert_batch(&self, docs: &[LanceDocument]) -> Result<()> {
        if docs.is_empty() {
            return Ok(());
        }

        // Convert to Arrow RecordBatch
        let record_batch = self.docs_to_record_batch(docs)?;
        let schema = record_batch.schema();
        let reader = Box::new(RecordBatchIterator::new(
            vec![Ok(record_batch)].into_iter(),
            schema,
        ));
        
        // Check if table exists, create if not
        if self.db.table_names().execute().await?.contains(&self.table_name) {
            // Append to existing table
            self.db.open_table(&self.table_name).execute().await?
                .add(reader).execute().await?;
        } else {
            // Create new table
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
            Field::new("vector", DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                1024
            ), true),
        ]));

        let mut ids = Vec::new();
        let mut doc_ids = Vec::new();
        let mut doc_paths = Vec::new();
        let mut categories = Vec::new();
        let mut category_texts = Vec::new();
        let mut contents = Vec::new();
        let mut chunk_indices = Vec::new();
        let mut total_chunks = Vec::new();
        let mut vectors: Vec<Option<Vec<Option<f32>>>> = Vec::new();

        for doc in docs {
            ids.push(doc.id.clone());
            doc_ids.push(doc.doc_id.clone());
            doc_paths.push(doc.doc_path.clone());
            categories.push(doc.category.clone());
            category_texts.push(doc.category_text.clone());
            contents.push(doc.content.clone());
            chunk_indices.push(doc.chunk_index as i32);
            total_chunks.push(doc.total_chunks as i32);
            vectors.push(Some(doc.vector.iter().map(|&x| Some(x)).collect()));
        }

        let record_batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(ids)),
                Arc::new(StringArray::from(doc_ids)),
                Arc::new(StringArray::from(doc_paths)),
                Arc::new(StringArray::from(categories)),
                Arc::new(StringArray::from(category_texts)),
                Arc::new(StringArray::from(contents)),
                Arc::new(Int32Array::from(chunk_indices)),
                Arc::new(Int32Array::from(total_chunks)),
                Arc::new(FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(
                    vectors.into_iter(),
                    1024,
                )),
            ],
        )?;

        Ok(record_batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_lancedb_indexer() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_db");

        let indexer = LanceDbIndexer::new(&db_path, "test_table").await.unwrap();

        // Test with empty chunks
        let chunks = vec![];
        let result = indexer.index_chunks(&chunks).await;
        assert!(result.is_ok());
    }
}
