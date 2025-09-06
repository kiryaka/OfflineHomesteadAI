use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ChunkId = String;
pub type Meta = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: ChunkId,
    pub doc_id: String,
    pub doc_path: String,
    pub category: String,
    pub category_text: String,
    pub content: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceKind {
    Vector,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: ChunkId,
    pub score: f32,
    pub source: SourceKind,
}

