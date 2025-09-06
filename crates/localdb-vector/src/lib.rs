use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use lancedb::{connect, Connection};
use lancedb::query::{QueryBase, ExecutableQuery};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use arrow_array::{RecordBatch, RecordBatchIterator, Int32Array, FixedSizeListArray, StringArray};
use arrow_schema::{Schema, Field, DataType};

use localdb_core::types::DocumentChunk;
use localdb_core::traits::Embedder;
use localdb_embed::get_default_embedder;

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

pub mod schema;
pub mod writer;
pub mod search;

pub use search::LanceSearchEngine;
pub use writer::LanceDbIndexer;
