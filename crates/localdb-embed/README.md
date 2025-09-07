# localdb-embed

Local embedding providers for the workspace.

- BGE‑M3 (XLM‑R) embedder via Candle + safetensors; FP16 on Metal/MPS, FP32 on CPU
- FakeEmbedder for tests and fast dev; enabled by `APP_USE_FAKE_EMBEDDINGS=1`

## Design & Responsibilities

- Load a local model (`model.safetensors`, `config.json`, `tokenizer.json`) from a configurable directory
- Provide batched tokenization on the target device/dtype
- Pool hidden states (masked mean + L2) and return normalized vectors
- Expose a simple `Embedder` trait through `get_default_embedder()` used across the codebase

## Modules (Files)

- `lib.rs`
  - `BgeM3Embedder` — safetensors loader; `embed_batch` on device
  - `FakeEmbedder` — deterministic, L2‑normalized vectors for tests
  - `get_default_embedder()` — switches to Fake if `APP_USE_FAKE_EMBEDDINGS=1`
- `device.rs` — device selection (Metal vs CPU)
- `tokenize.rs` — `tokenize_batch_on_device` (ids & attention mask on device/dtype)
- `pool.rs` — `masked_mean_l2(hidden, attn)` with dtype‑safe broadcasting
- `tests/pool_tests.rs` — unit tests for pooling

## Configuration

Model directory is discovered by the following in order:

1. `APP_MODEL_DIR`
2. `MODEL_DIR`
3. `../models/bge-m3` (workspace relative)
4. `models/bge-m3`

Full safetensors path must contain:
- `model.safetensors`, `config.json`, `tokenizer.json` (HF layout)

## Quick Start

```rust
use localdb_embed::get_default_embedder;
let emb = get_default_embedder()?; // Fake if APP_USE_FAKE_EMBEDDINGS=1
let vecs = emb.embed_batch(&["hello".to_string(), "world".to_string()])?;
assert_eq!(vecs[0].len(), emb.dim());
```

## Notes

- The fake embedder is extremely fast and ideal for tests and dev flows.
- Real model loads happen once; subsequent calls are batched and run on the selected device.

