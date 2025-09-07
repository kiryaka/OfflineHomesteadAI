pub mod tantivy_utils;
pub mod index;
pub mod search;

pub use index::TantivyIndexer;
pub use search::{TantivySearchEngine, SearchResult};
//! localdb-text
//!
//! Tantivy-based text indexing and search. See `index` and `search` modules and
//! examples under `examples/` for CLI-like usage during development.
