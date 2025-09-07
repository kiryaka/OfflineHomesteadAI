//! Trait surfaces for pluggable engines and embedders.

use crate::types::{DocumentChunk, SearchHit};

/// Produces L2-normalized embedding vectors for input text.
pub trait Embedder: Send + Sync {
    fn dim(&self) -> usize;
    fn max_len(&self) -> usize;
    fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>>;
}

/// Indexes and searches the text corpus (e.g., Tantivy/BM25).
pub trait TextIndexer: Send + Sync {
    fn index(&self, chunks: &[DocumentChunk]) -> anyhow::Result<()>;
    fn search(&self, query: &str, k: usize) -> anyhow::Result<Vec<SearchHit>>;
}

/// Indexes and searches vector embeddings (e.g., Lance IVF_PQ).
pub trait VectorIndexer: Send + Sync {
    fn index(&self, chunks: &[DocumentChunk], embeddings: &[Vec<f32>]) -> anyhow::Result<()>;
    fn search_vec(&self, query_vec: &[f32], k: usize) -> anyhow::Result<Vec<SearchHit>>;
}

/// Façade for a combined engine that exposes a unified interface.
pub trait SearchEngine: Send + Sync {
    fn index(&self, chunks: &[DocumentChunk]) -> anyhow::Result<()>;
    fn query(&self, query: &str, k: usize) -> anyhow::Result<Vec<SearchHit>>;
}
