//! Domain types used by text and vector engines.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ChunkId = String;
pub type Meta = HashMap<String, String>;

/// A chunk of a source document that is independently indexed.
///
/// - `id`: globally unique chunk identifier
/// - `doc_id`: stable document identity (file stem or external id)
/// - `doc_path`: original path to the source file
/// - `category`/`category_text`: hierarchical facet (e.g., "/topic/subtopic")
/// - `content`: the text payload of the chunk
/// - `chunk_index`/`total_chunks`: position within the parent document
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

/// Indicates which engine produced a result.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceKind {
    Vector,
    Text,
}

/// The minimal surface returned by all engines.
///
/// `id` matches `DocumentChunk::id`. `score` is engine-specific but
/// higher is always better. `source` labels the origin engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: ChunkId,
    pub score: f32,
    pub source: SourceKind,
}
