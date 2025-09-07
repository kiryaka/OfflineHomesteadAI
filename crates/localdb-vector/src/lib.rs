pub mod schema;
pub mod table;
pub mod embed_provider;
pub mod cache;
pub mod embed_backfill;
pub mod index_build;
pub mod writer;
pub mod search;

pub use search::LanceSearchEngine;
pub use writer::LanceDbIndexer;
//! localdb-vector
//!
//! Lance/LanceDB-based vector pipeline with side-table embeddings, first-class
//! caching, status-driven backfill, and atomic index builds. See the crate
//! README for a full design overview and examples under `examples/` for
//! development workflows.
