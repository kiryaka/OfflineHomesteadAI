# localdb-vector

Vector indexing and search for the OfflineHomesteadAI workspace, implemented on top of Lance/LanceDB.

This crate provides a resumable, testable, and modular pipeline for building vector indexes over chunked documents. It is designed for clean separation of concerns, pluggable embedders, and safe operation on large corpora.

## Design Goals

- Resumable ingestion: never start from scratch; make progress in small batches.
- Side-table embeddings: keep training scans small and avoid poisoning serving data.
- First-class cache: avoid re-embedding identical content; work offline when possible.
- Atomic index build: build under a new ID, validate, then flip the active pointer.
- Pluggable providers: local (safetensors) and remote (e.g., Novita AI) with rate-limit guards.
- Testability: fast fake embedders and in-memory/temp DBs for CI.

## Data Model

We use Lance tables (via LanceDB). There are three core tables plus a simple key/value meta table.

- `documents` (serving + pipeline state)
  - `id: Utf8` (unique)
  - `doc_id: Utf8`, `doc_path: Utf8`
  - `category: Utf8`, `category_text: Utf8`
  - `content: Utf8`
  - `chunk_index: Int32`, `total_chunks: Int32`
  - `vector: FixedSizeList<Float32, D=1024>` (nullable; the serving column)
  - `content_hash: Utf8` (blake3 of `content`)
  - `embedding_status: Utf8` ∈ {`new`,`in_progress`,`ready`,`error`}
  - `embedding_error: Utf8?` (last error string if any)
  - `embedding_version: Int32` (monotonic per-row)
  - `embedded_at: Timestamp(ms)?`
  - `index_status: Utf8` (reserved; currently `stale`/`ready`)
  - `index_version: Int32`

- `embeddings` (side-table; training/AB source)
  - `id: Utf8` (chunk id)
  - `embedder_id: Utf8` (e.g., `local:...:d1024`)
  - `content_hash: Utf8`
  - `embedded_at: Timestamp(ms)`
  - `vector: FixedSizeList<Float32, 1024>`

- `emb_cache` (first-class cache)
  - `content_hash: Utf8`
  - `embedder_id: Utf8`
  - `created_at: Timestamp(ms)`
  - `vector: FixedSizeList<Float32, 1024>`

- `meta` (K/V control table)
  - `key: Utf8`, `value: Utf8`, `updated_at: Timestamp(ms)`
  - Used for e.g., `active_index_id:documents` pointer.

### Status Transitions

- Embedding stage (`embedding_status`):
  - `new → in_progress → ready` on success
  - `new → in_progress → error` on failure (with `embedding_error` set)
  - Retries move `error → in_progress` then either `ready` or `error` again.

- Index stage (`index_status`):
  - Currently set to `stale` on ingest; advanced orchestration will set training/build/ready soon.

## Modules (Files)

- `schema.rs` — Arrow schemas for all tables; `EMBEDDING_DIM` constant.
- `table.rs` — LanceDB helpers:
  - `open_db(uri)`, `ensure_embeddings_table(...)`, `ensure_cache_table(...)`
  - `ensure_meta_table`, `set_meta`, `get_meta` (simple K/V control)
- `writer.rs` — Ingestion helper for `documents`.
  - Fills `content_hash`, status/version fields; `vector` optional (serving only).
- `embed_provider/` — Embedding provider abstraction.
  - `mod.rs` — `trait EmbedProvider { embedder_id, dim, max_len, embed_batch }`
  - `local.rs` — Local provider using the safetensors-backed BGE‑M3 embedder from `localdb-embed`.
- `cache.rs` — First-class cache API for `(content_hash, embedder_id) → vector` (Lance-backed).
- `embed_backfill.rs` — Resumable backfill loop:
  - Selects non‑ready rows; marks `in_progress`; reads cache; embeds misses; writes to `embeddings` + cache; marks `ready`.
- `index_build.rs` — Training/build/flip scaffolding:
  - `compute_ivfpq_params(total_ready, dim)` — sensible defaults with clamps for tiny datasets
  - `sync_serving_vectors_from_embeddings` — copies side-table vectors into `documents.vector` via merge_insert
  - `build_ivfpq_index` — constructs an IVF_PQ index on `vector` with a custom name
  - `validate_index` — sanity check (non-empty top‑k on a small sample)
  - `flip_active_index` — stores `active_index_id:<table>` in `meta`
- `search.rs` — (existing) basic search helpers

## Quick Start (Examples)

These are convenience examples for local poking. For CI/fast tests, see the test section.

Assume workspace root and dev_data present. Use fake embeddings for speed:

```
export APP_USE_FAKE_EMBEDDINGS=1
# 1) Initialize documents (vector=null, status=new)
cargo run -p localdb-vector --example init_chunks
# 2) Backfill embeddings (local provider)
cargo run -p localdb-vector --example backfill
# 3) Inspect counts
cargo run -p localdb-vector --example status
# 4) Train + build IVF_PQ index; validate and flip active pointer
cargo run -p localdb-vector --example train_build
```

Notes:
- Examples resolve paths relative to the workspace root.
- Use a small subset if your corpus is large.

## Tests

Fast integration test (uses a temporary Lance DB and fake embeddings):

- `crates/localdb-vector/tests/pipeline_tests.rs`
  - Seeds ~300 synthetic chunks into `documents`.
  - Runs backfill → sync serving vectors → computes params → builds index → validates → flips active pointer.
  - Run: `APP_USE_FAKE_EMBEDDINGS=1 cargo test -p localdb-vector --tests`

To make tests faster, we clamp PQ params for tiny datasets. For non-trivial datasets, PQ training will be CPU-bound and multi-threaded (expected).

## Embedding Providers

- LocalProvider (default):
  - Safetensors-backed BGE‑M3 model loaded via `localdb-embed`; batch tokenization; FP16 on Metal / FP32 on CPU.
  - Use `APP_USE_FAKE_EMBEDDINGS=1` to switch to the FakeEmbedder in dev and tests.

- Remote (Novita) — planned:
  - Rate limiting (token bucket) + per-host concurrency cap
  - Retries with backoff + jitter for 429/5xx; circuit breaker on consecutive failures
  - Daily spend cap; warm cache before remote calls

## Indexing Strategy

- Metric: cosine with L2-normalized vectors (embedder outputs normalized vectors).
- IVF_PQ defaults for large N:
  - `nlist = min(max(2048, 2*sqrt(N)), 65536)`
  - `m = 32` for 1024‑dim (else `m = 16`)
  - `nbits = 8`
- Small N handling:
  - Clamp `nlist ≤ max(1, N-1)` to avoid invalid KMeans.
  - PQ requires ≥ 256 training rows; for smaller sets, consider `Index::Auto` or tiny params in test flows.

## Pipeline Flow (Resumable)

1) Ingest chunks into `documents` (vector=null; status=new)
2) Build scalar indexes (category/doc_path) — TODO (scaffolded)
3) Backfill embeddings (select `new`/`error`):
   - Mark `in_progress`
   - Cache lookup; embed misses; write to `embeddings` + `emb_cache`
   - Mark `ready`, set timestamps and bump versions
4) Train/build index:
   - Sample from `embeddings` for the target `embedder_id`
   - Compute params; build IVF_PQ named index
   - Validate; flip active index pointer in `meta`

## Remember / Gotchas

- Don’t run dev examples on huge corpora unintentionally; use the tests or a small subset.
- Backfill is status-driven and idempotent; re-running skips `ready` rows.
- Serving column `documents.vector` is only synced from side-table during build/swap (keeps serving clean during backfill).
- Active index pointer is stored in `meta`; search can read and use it if needed (currently Lance uses whichever index is present on the column).
- Cache is first-class — we always check it before embedding.

## Roadmap / TODO

- Remote provider (Novita) with rate limiting + backoff + daily cap
- Scalar index builders (category/path) for faster filters
- Predicate pushdown in cache queries to avoid full scans
- Better validation (held-out recall@k vs flat baseline; thresholds)
- Sharding: introduce a shard dimension (e.g., top-level category) and replicate the pattern per shard
- Richer status/index lifecycle (`stale`/`training`/`building`/`ready`/`error`)
- Query layer that reads `meta.active_index_id` to choose index-specific settings (e.g., nprobe)

## Configuration

- `APP_USE_FAKE_EMBEDDINGS=1` — use fake embedders (fast, deterministic) in dev/tests
- Workspace deps unified in root `Cargo.toml` ([workspace.dependencies]); sccache configured in `.cargo/config.toml`

## Performance Hints

- Use `sccache` for compiler caching; keep `CARGO_INCREMENTAL` on for local crates and off for deps (profile settings included).
- For tiny tests, prefer lightweight index params or `Index::Auto`.
- For large builds, tune `nprobe` (e.g., 32–64) at query time to balance recall/latency.

---

This README reflects the current implementation and near-term plan. If you spot drift, please update this document alongside code changes so the design stays discoverable.
