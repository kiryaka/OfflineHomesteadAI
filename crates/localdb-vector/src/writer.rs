use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use lancedb::{connect, Connection};
use arrow_array::{RecordBatch, RecordBatchIterator, Int32Array, FixedSizeListArray, StringArray};
use std::sync::Arc;
use std::path::Path;

use localdb_core::types::DocumentChunk;
use crate::schema::{build_arrow_schema, EMBEDDING_DIM};
use localdb_embed::get_default_embedder;

#[derive(Debug, Clone)]
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

pub struct LanceDbIndexer { pub(crate) db: Connection, pub(crate) table_name: String }

impl LanceDbIndexer {
	pub async fn new(db_path: &Path, table_name: &str) -> Result<Self> {
		let db = connect(db_path.to_string_lossy().as_ref()).execute().await?;
		Ok(Self { db, table_name: table_name.to_string() })
	}

	pub async fn index(&self, chunks: &[DocumentChunk], embeddings: &[Vec<f32>]) -> Result<()> {
		if chunks.is_empty() { println!("No chunks to index"); return Ok(()); }
		assert_eq!(chunks.len(), embeddings.len(), "chunks and embeddings length must match");
		println!("Indexing {} chunks into LanceDB table: {}", chunks.len(), self.table_name);
		let pb = ProgressBar::new(chunks.len() as u64);
		pb.set_style(ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} chunks ({percent}%) {msg}").unwrap().progress_chars("#>-") );
		let mut processed = 0usize; let mut batch_docs = Vec::new(); let batch_size = 1000usize;
		for (i, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
			let doc = LanceDocument { id: chunk.id.clone(), doc_id: chunk.doc_id.clone(), doc_path: chunk.doc_path.clone(), category: chunk.category.clone(), category_text: chunk.category_text.clone(), content: chunk.content.clone(), chunk_index: chunk.chunk_index, total_chunks: chunk.total_chunks, vector: embedding.clone() };
			batch_docs.push(doc); processed += 1; pb.set_position(processed as u64); pb.set_message(format!("Processing chunk {}", i + 1));
			if batch_docs.len() >= batch_size || i == chunks.len() - 1 { self.insert_batch(&batch_docs).await?; batch_docs.clear(); if processed % 1000 == 0 { println!("\n📦 Processed batch of 1000 chunks..."); } }
		}
		pb.finish_with_message("✅ LanceDB indexing completed!");
		println!("📊 Successfully indexed {} chunks into LanceDB", processed);
		Ok(())
	}

	pub async fn index_chunks(&self, chunks: &[DocumentChunk]) -> Result<()> {
		if chunks.is_empty() { return Ok(()); }
		let embedder = get_default_embedder()?;
		let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
		let embeddings = embedder.embed_batch(&texts)?;
		self.index(chunks, &embeddings).await
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
		let schema = build_arrow_schema();
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
            Arc::new(FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(vectors.into_iter(), EMBEDDING_DIM)),
        ])?;
		Ok(record_batch)
	}
}
