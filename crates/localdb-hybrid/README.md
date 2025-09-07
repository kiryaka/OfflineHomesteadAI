# localdb-hybrid

Hybrid search façade that composes a text indexer and a vector indexer behind one `SearchEngine` trait.

## Design & Responsibilities

- Present a unified `index` and `query` interface
- Embed the query once, query both engines, and merge/dedupe results by id
- Label each hit with `SourceKind::{Text, Vector}` for downstream logic

## Core Type

`HybridSearchEngine<TI, VI>` where `TI: TextIndexer`, `VI: VectorIndexer`:

- `index(&[DocumentChunk])`:
  - Batch embed chunks via the provided `Embedder`
  - `vector.index(chunks, embeddings)` then `text.index(chunks)`
- `query(&str, k)`:
  - Embed query, collect `vector.search_vec(q, k)` and `text.search(q, k)`
  - Merge by id, keep higher score on conflict, sort and truncate to `k`

## Usage

```rust
use localdb_hybrid::HybridSearchEngine;
use localdb_core::traits::{TextIndexer, VectorIndexer, Embedder};

fn make_engine(text: impl TextIndexer, vector: impl VectorIndexer, emb: Box<dyn Embedder>) -> impl localdb_core::traits::SearchEngine {
    HybridSearchEngine::new(text, vector, emb)
}
```

## Notes

- The hybrid layer is intentionally thin: it delegates heavy lifting to the underlying text/vector crates.
- Scoring merge is naive (take max score by id); add a refined reranker as needed.

