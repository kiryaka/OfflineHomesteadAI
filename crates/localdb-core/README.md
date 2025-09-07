# localdb-core

Core types, traits, configuration, and chunking utilities shared across the workspace.

## Design & Responsibilities

- Define the core domain model used by the text and vector engines.
- Provide trait surfaces so engines are pluggable and testable.
- Offer a light Figment-based configuration layer.
- Include a pragmatic text chunker for `.txt` sources.

## Modules (Files)

- `types.rs`
  - `DocumentChunk` — the unit of indexing (id, doc_id, doc_path, category, content, chunk_index, total_chunks)
  - `SearchHit` — a hit id + score + `SourceKind` (`Text` or `Vector`)
  - `SourceKind` — where a hit came from
- `traits.rs`
  - `Embedder` — `dim`, `max_len`, `embed_batch(&[String]) -> Vec<Vec<f32>>`
  - `TextIndexer` — `index(&[DocumentChunk])`, `search(&str, k)` → `Vec<SearchHit>`
  - `VectorIndexer` — `index(&[DocumentChunk], &[Vec<f32>])`, `search_vec(&[f32], k)` → `Vec<SearchHit>`
  - `SearchEngine` — unified `index/query` façade
- `config.rs`
  - `Config::load()` via Figment (toml + env `APP_*`); `expand_path`, `resolve_with_base`
- `data_processor.rs`
  - `DataProcessor` — chunk a directory of `.txt` into `DocumentChunk`s, paragraph‑based with overlap
- `error.rs` — typed error wrapper (`thiserror`)
- `lib.rs` — glues the above, denies warnings in this crate

## Quick Start

Chunk a folder of `.txt` into `DocumentChunk`s:

```rust
use localdb_core::data_processor::DataProcessor;
let chunks = DataProcessor::new().process_directory(std::path::Path::new("dev_data/txt"))?;
```

Implement a custom embedder (for tests):

```rust
struct MyEmbed;
impl localdb_core::traits::Embedder for MyEmbed {
    fn dim(&self) -> usize { 8 }
    fn max_len(&self) -> usize { 64 }
    fn embed_batch(&self, texts:&[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|_| vec![0.0; 8]).collect())
    }
}
```

## Notes

- All traits are `Send + Sync` to support async runtimes and parallelism.
- Chunking is heuristic and optimized for simplicity; adjust `DataProcessor` as needed for your corpus.

## Roadmap / TODO

- Strongly‑typed config (structs + validation) on top of Figment
- Additional helpers for content normalization and language detection
