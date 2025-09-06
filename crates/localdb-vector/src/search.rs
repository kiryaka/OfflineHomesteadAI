use anyhow::Result;
use futures::TryStreamExt;
use lancedb::{connect, Connection};
use lancedb::query::{QueryBase, ExecutableQuery};
use localdb_core::traits::Embedder;
use localdb_embed::get_default_embedder;
use localdb_core::traits::VectorIndexer;
use localdb_core::types::{DocumentChunk, SearchHit, SourceKind};

pub struct LanceSearchEngine { pub(crate) db: Connection, pub(crate) table_name: String, pub(crate) embedder: Box<dyn Embedder> }

impl LanceSearchEngine {
	pub async fn new(db_path: std::path::PathBuf, table_name: &str) -> Result<Self, anyhow::Error> {
		let embedder = get_default_embedder()?; let db = connect(db_path.to_string_lossy().as_ref()).execute().await?;
		Ok(Self { db, table_name: table_name.to_string(), embedder })
	}

	pub async fn search(&self, query_text: &str, limit: usize) -> Result<Vec<LanceSearchResult>, anyhow::Error> {
		let query_embedding = self.embedder.embed_batch(&[query_text.to_string()])?.remove(0); let table = self.db.open_table(&self.table_name).execute().await?;
		let pq_limit = limit * 10; let mut results = table.vector_search(query_embedding)?.limit(pq_limit).execute().await?;
		let mut all_results = Vec::new();
		while let Some(batch) = TryStreamExt::try_next(&mut results).await? {
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

impl VectorIndexer for super::writer::LanceDbIndexer {
	fn index(&self, chunks: &[DocumentChunk], embeddings: &[Vec<f32>]) -> anyhow::Result<()> {
		// This type currently exposes async index; for trait compatibility we block here.
		let rt = tokio::runtime::Runtime::new()?;
		rt.block_on(async { self.index(chunks, embeddings).await })
	}
	fn search_vec(&self, q_vec: &[f32], k: usize) -> anyhow::Result<Vec<SearchHit>> {
		let rt = tokio::runtime::Runtime::new()?;
		let table = rt.block_on(async { self.db.open_table(&self.table_name).execute().await })?;
		let mut stream = rt.block_on(async { table.vector_search(q_vec.to_vec())?.limit(k).execute().await })?;
		let mut hits = Vec::new();
		while let Some(batch) = rt.block_on(async { TryStreamExt::try_next(&mut stream).await })? {
			for i in 0..batch.num_rows() {
				let id = batch.column_by_name("id").unwrap().as_any().downcast_ref::<arrow_array::StringArray>().unwrap().value(i).to_string();
				let score = if let Some(distance_col) = batch.column_by_name("_distance") { 1.0 - distance_col.as_any().downcast_ref::<arrow_array::Float32Array>().unwrap().value(i) } else { 0.5 };
				hits.push(SearchHit { id, score, source: SourceKind::Vector });
			}
		}
		Ok(hits)
	}
}

#[derive(Debug, Clone)]
pub struct LanceSearchResult { pub score: f32, pub id: String, pub category: String, pub path: String, pub content: String }
