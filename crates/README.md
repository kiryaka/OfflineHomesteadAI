# Rust Crates Overview

This repository’s Rust workspace is organized into focused crates that compose into a complete local, offline‑first search stack:

- localdb-core: shared types, traits, config, and chunking utilities
- localdb-embed: local (safetensors) embedder + fake embedder for tests
- localdb-text: Tantivy‑based text indexing and search
- localdb-vector: Lance/LanceDB‑based vector pipeline (resumable, cached, atomic index build)
- localdb-hybrid: a façade that merges text + vector results behind one SearchEngine

Read each crate’s README for details. This page summarizes the big picture and how the parts fit together.

## Big Picture

1) Ingest: chunk raw text into DocumentChunk and persist into Lance `documents` (no vectors yet)
2) Text index: build Tantivy index over chunks for BM25 search and faceting
3) Vector pipeline (Lance):
   - Backfill embeddings with a provider (local or remote) into a side `embeddings` table, caching by (content_hash, embedder_id)
   - Train and build an IVF_PQ index, validate, then atomically flip the active index pointer
4) Hybrid search: embed the query once, get top‑k from vector + top‑k from text, merge/dedupe by id with source labelling

## Crates At A Glance

- localdb-core
  - Types: `DocumentChunk`, `SearchHit`, `SourceKind`
  - Traits: `Embedder`, `TextIndexer`, `VectorIndexer`, `SearchEngine`
  - Config: Figment‑based; `Config::load()`; `expand_path`, `resolve_with_base`
  - Utilities: `DataProcessor` to chunk a directory of `.txt`

- localdb-embed
  - BGE‑M3 (XLM‑R) embedder via Candle + safetensors; FP16 on Metal/MPS, FP32 on CPU
  - Fake embedder: deterministic, fast; `APP_USE_FAKE_EMBEDDINGS=1`
  - Batched tokenization on device/dtype; masked‑mean + L2 pooling

- localdb-text
  - Tantivy indexer/searcher; custom tokenizer setup; BM25 search with snippet + facets
  - Examples: `examples/index.rs` (reindex), `examples/search.rs` (query)

- localdb-vector
  - Lance/LanceDB documents + side tables; status‑driven backfill; first‑class cache
  - IVF_PQ training/build with named index IDs; minimal validation; active index pointer in meta
  - Examples: `init_chunks`, `backfill`, `status`, `train_build`
  - Test: pipeline test using a temp DB and fake embedders

- localdb-hybrid
  - `HybridSearchEngine<TI,VI>` merges text + vector by id; sets `SourceKind` for each hit; simple, composable façade

## Development Flow

- Prefer tests for end‑to‑end validation (fast, controlled) over running examples on the full corpus.
- Use the fake embedder in dev/CI: `export APP_USE_FAKE_EMBEDDINGS=1`.
- sccache is enabled via `.cargo/config.toml` for faster rebuilds.

## Quick Start (dev)

- Rebuild Tantivy over dev_data:
  - `cargo run -p localdb-text --example index -- --dir dev_data/txt --index dev_data/indexes/tantivy`
- Search:
  - `cargo run -p localdb-text --example search -- "your query" --index dev_data/indexes/tantivy`
- Lance vector pipeline (small subset recommended):
  - `cargo run -p localdb-vector --example init_chunks`
  - `cargo run -p localdb-vector --example backfill`
  - `cargo run -p localdb-vector --example train_build`

## Roadmap

- Remote embedding provider (Novita): rate limiting, retries, daily cost cap, cache warm‑before‑remote
- Scalar index builders (category/path) in Lance for faster filters
- Better validation (held‑out recall@k vs flat baseline), auto‑tune nprobe against latency targets
- Sharding (by top‑level category) as data grows
