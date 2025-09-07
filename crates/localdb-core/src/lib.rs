#![deny(warnings)]
#![deny(dead_code)]
#![deny(unused_variables)]
#![deny(unused_imports)]
//! localdb-core
//!
//! Core types, traits, config helpers, and chunking utilities shared across the
//! workspace. This crate defines the domain model (`DocumentChunk`), the primary
//! trait surfaces (`Embedder`, `TextIndexer`, `VectorIndexer`, `SearchEngine`),
//! and a pragmatic `DataProcessor` for turning a directory of `.txt` files into
//! chunks suitable for indexing.
//!
//! The documentation of each module provides more details.

pub mod config;
pub mod data_processor;
pub mod error;
pub mod traits;
pub mod types;
