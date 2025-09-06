use crate::types::{DocumentChunk, SearchHit};

pub trait Embedder: Send + Sync {
    fn dim(&self) -> usize;
    fn max_len(&self) -> usize;
    fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>>;
}

pub trait TextIndexer: Send + Sync {
    fn index(&self, chunks: &[DocumentChunk]) -> anyhow::Result<()>;
    fn search(&self, query: &str, k: usize) -> anyhow::Result<Vec<SearchHit>>;
}

pub trait VectorIndexer: Send + Sync {
    fn index(&self, chunks: &[DocumentChunk], embeddings: &[Vec<f32>]) -> anyhow::Result<()>;
    fn search_vec(&self, query_vec: &[f32], k: usize) -> anyhow::Result<Vec<SearchHit>>;
}

pub trait SearchEngine: Send + Sync {
    fn index(&self, chunks: &[DocumentChunk]) -> anyhow::Result<()>;
    fn query(&self, query: &str, k: usize) -> anyhow::Result<Vec<SearchHit>>;
}

