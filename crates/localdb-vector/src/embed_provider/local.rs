use anyhow::Result;
use localdb_core::traits::Embedder as CoreEmbedder;
use localdb_embed::get_default_embedder;

use super::EmbedProvider;

pub struct LocalProvider {
    inner: Box<dyn CoreEmbedder>,
    id: String,
}

impl LocalProvider {
    /// Create a new local provider, loading the default embedder.
    pub fn new() -> Result<Self> {
        let inner = get_default_embedder()?;
        let id = format!("local:{}:d{}", std::any::type_name::<Self>(), inner.dim());
        Ok(Self { inner, id })
    }
}

impl EmbedProvider for LocalProvider {
    fn embedder_id(&self) -> &str { &self.id }
    fn dim(&self) -> usize { self.inner.dim() }
    fn max_len(&self) -> usize { self.inner.max_len() }
    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> { self.inner.embed_batch(texts) }
//! Local embedding provider using the crate `localdb-embed`.
//!
//! Respects `APP_USE_FAKE_EMBEDDINGS=1` to switch to the FakeEmbedder for fast
//! and deterministic outputs in tests and development.
}
