// Library crate for tantivy_demo
// Exposes modules for use in integration tests

#![deny(warnings)]
#![deny(dead_code)]
#![deny(unused_variables)]
#![deny(unused_imports)]

pub mod config;
pub mod data_processor;
pub mod embedding;
pub mod lancedb_indexer;
pub mod tantivy_utils;
pub mod lance_utils;
