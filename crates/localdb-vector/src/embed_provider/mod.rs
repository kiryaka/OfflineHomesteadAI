use anyhow::Result;

pub trait EmbedProvider: Send + Sync {
    /// Stable identifier for the provider/model (e.g., `local:...:d1024`).
    fn embedder_id(&self) -> &str;
    /// Embedding dimensionality (D).
    fn dim(&self) -> usize;
    /// Maximum token length for this provider.
    fn max_len(&self) -> usize;
    /// Compute embeddings for a batch of input texts.
    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
//! Embedding provider abstraction used by the backfill pipeline.
//!
//! Implementations may call a local model (see `local.rs`) or a remote API
//! (planned). Providers must return L2‑normalized vectors of the same
//! dimensionality for a given `embedder_id`.
}

pub mod local;
// pub mod novita; // to be added later
