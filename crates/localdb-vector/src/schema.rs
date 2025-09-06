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
		Field::new("vector", DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), EMBEDDING_DIM), true),
	]))
}