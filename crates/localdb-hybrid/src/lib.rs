use anyhow::Result;
use localdb_core::traits::{Embedder, TextIndexer, VectorIndexer, SearchEngine};
use localdb_core::types::{DocumentChunk, SearchHit, SourceKind};

pub struct HybridSearchEngine<TI, VI> where TI: TextIndexer, VI: VectorIndexer {
    text: TI,
    vector: VI,
    embedder: Box<dyn Embedder>,
}

impl<TI, VI> HybridSearchEngine<TI, VI> where TI: TextIndexer, VI: VectorIndexer {
    pub fn new(text: TI, vector: VI, embedder: Box<dyn Embedder>) -> Self { Self { text, vector, embedder } }

    pub fn index(&self, chunks: &[DocumentChunk]) -> Result<()> {
        // 1) embed in batches
        let batch_texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedder.embed_batch(&batch_texts)?;
        for e in &embeddings { assert_eq!(e.len(), self.embedder.dim()); }
        // 2) vector index
        self.vector.index(chunks, &embeddings)?;
        // 3) text index
        self.text.index(chunks)
    }

    pub fn query(&self, query: &str, k: usize) -> Result<Vec<SearchHit>> {
        let q_vec = self.embedder.embed_batch(&[query.to_string()])?.remove(0);
        let mut dense_hits = self.vector.search_vec(&q_vec, k)?;
        for h in &mut dense_hits { h.source = SourceKind::Vector; }
        let mut text_hits = self.text.search(query, k)?;
        for h in &mut text_hits { h.source = SourceKind::Text; }
        // merge unique ids, prioritize better score
        use std::collections::HashMap;
        let mut by_id: HashMap<String, SearchHit> = HashMap::new();
        for h in dense_hits.into_iter().chain(text_hits.into_iter()) {
            by_id.entry(h.id.clone()).and_modify(|old| { if h.score > old.score { *old = h.clone(); } }).or_insert(h);
        }
        let mut merged: Vec<SearchHit> = by_id.into_values().collect();
        merged.sort_by(|a,b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        merged.truncate(k);
        Ok(merged)
    }
}

impl<TI, VI> SearchEngine for HybridSearchEngine<TI, VI> where TI: TextIndexer, VI: VectorIndexer {
    fn index(&self, chunks: &[DocumentChunk]) -> Result<()> { Self::index(self, chunks) }
    fn query(&self, query: &str, k: usize) -> Result<Vec<SearchHit>> { Self::query(self, query, k) }
}
