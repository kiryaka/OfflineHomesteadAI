# localdb-text

Text indexing and search using Tantivy (BM25), with snippets and facet counts.

## Design & Responsibilities

- Index `DocumentChunk` content into a Tantivy index
- Provide fast BM25 search with snippets and facet aggregation
- Offer a small hybrid-friendly API (id, path, category, snippet, score)

## Modules (Files)

- `index.rs` — create/rebuild index from a directory or chunk stream
- `search.rs` — BM25 search with AND/phrase boosting; facet counts
- `tantivy_utils.rs` — tokenizer/analysis setup and schema helpers
- `lib.rs` — re-exports and wiring
- `examples/index.rs` — reindex a directory (defaults to workspace dev paths)
- `examples/search.rs` — query and print results (with optional facets)

## Quick Start (dev)

```bash
# Reindex from dev_data/txt
env RUST_LOG=info cargo run -p localdb-text --example index -- --dir dev_data/txt --index dev_data/indexes/tantivy
# Search
cargo run -p localdb-text --example search -- "paper casings" --index dev_data/indexes/tantivy
```

## Query Semantics

- Default OR (BM25)
- Additional boosted subqueries in `search.rs`:
  - AND‑by‑default version of the query (boost ×2)
  - Exact phrase query if multiword (boost ×4)
- Combined with a Boolean SHOULD query so strict matches rank higher but OR matches still appear

## Notes

- Build time for the index depends on corpus size; use a subset when exploring.
- BM25 tuning (k1, b) can be added if needed.

