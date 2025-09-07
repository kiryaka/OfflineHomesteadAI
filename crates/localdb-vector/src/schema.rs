use arrow_schema::{Schema, Field, DataType};
use std::sync::Arc;

pub const EMBEDDING_DIM: i32 = 1024;

pub fn build_arrow_schema() -> Arc<Schema> {
	Arc::new(Schema::new(vec![
		Field::new("id", DataType::Utf8, false),
		Field::new("doc_id", DataType::Utf8, false),
		Field::new("doc_path", DataType::Utf8, false),
		Field::new("category", DataType::Utf8, false),
		Field::new("category_text", DataType::Utf8, false),
		Field::new("content", DataType::Utf8, false),
		Field::new("chunk_index", DataType::Int32, false),
		Field::new("total_chunks", DataType::Int32, false),
		// Serving vector column (nullable); filled only after validation/build
		Field::new("vector", DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), EMBEDDING_DIM), true),
		// Resumability & index status
		Field::new("content_hash", DataType::Utf8, false),
		Field::new("embedding_status", DataType::Utf8, false),
		Field::new("embedding_error", DataType::Utf8, true),
		Field::new("embedding_version", DataType::Int32, false),
		Field::new("embedded_at", DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, None), true),
		Field::new("index_status", DataType::Utf8, false),
		Field::new("index_version", DataType::Int32, false),
	]))
}

pub fn build_embeddings_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("embedder_id", DataType::Utf8, false),
        Field::new("content_hash", DataType::Utf8, false),
        Field::new("embedded_at", DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, None), false),
        Field::new(
            "vector",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), EMBEDDING_DIM),
            true,
        ),
    ]))
}

pub fn build_cache_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("content_hash", DataType::Utf8, false),
        Field::new("embedder_id", DataType::Utf8, false),
        Field::new("created_at", DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, None), false),
        Field::new(
            "vector",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), EMBEDDING_DIM),
            true,
        ),
    ]))
}
//! Arrow schema builders for Lance tables used by the vector pipeline.
//!
//! Includes `documents` (serving + status), `embeddings` (side table for
//! training/AB), and `emb_cache` (first-class cache).
