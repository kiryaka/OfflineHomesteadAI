use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::xlm_roberta::{XLMRobertaModel, Config as XLMRobertaConfig};
use tokenizers::Tokenizer;

use localdb_core::traits::Embedder as CoreEmbedder;

mod device;
mod pool;
mod tokenize;

pub use device::*;
pub use pool::*;
pub use tokenize::*;

pub struct BgeM3Embedder { model: XLMRobertaModel, tokenizer: Tokenizer, device: Device, dtype: DType }

impl BgeM3Embedder {
    pub fn new() -> Result<Self> {
        let device = select_device();
        let dtype = match &device { Device::Metal(_) => DType::F16, _ => DType::F32 };
        println!("🔄 Loading BGE-M3 (XLM-R) from local files... device={:?} dtype={:?}", device, dtype);
        let model_dir = resolve_model_dir()?;
        let tokenizer_path = model_dir.join("tokenizer.json");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer from {}: {}", tokenizer_path.display(), e))?;
        let config_path = model_dir.join("config.json");
        let config: XLMRobertaConfig = serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;
        // Safetensors only: fail fast if missing
        let st = model_dir.join("model.safetensors");
        if !st.exists() { return Err(anyhow!("{} not found", st.display())); }
        // Safety: relying on safetensors metadata
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[st.to_string_lossy().into_owned()], dtype, &device)? };
        let model = XLMRobertaModel::new(&config, vb)?;
        Ok(Self { model, tokenizer, device, dtype })
    }

    /// Embed a single string (debug / one-off calls). Prefer `embed_batch`.
    #[allow(dead_code)]
    fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let start = Instant::now();
        let max_len = self.max_len();
        let (input_ids, attention_mask) = tokenize_on_device(&self.tokenizer, text, max_len, &self.device)?;
        // XLM‑R in candle-transformers expects a token_type_ids tensor; use zeros.
        let token_type_ids = Tensor::zeros((1, max_len), DType::I64, &self.device)?;
        let hidden_states = self.model.forward(&input_ids, &attention_mask, &token_type_ids, None, None, None)?;
        let embedding = masked_mean_l2(&hidden_states, &attention_mask)?;
        let emb_cpu = embedding.to_device(&Device::Cpu)?.to_vec1()?;
        assert_eq!(emb_cpu.len(), self.dim());
        if start.elapsed().as_millis() > 100 { println!("⚠️  Slow embedding"); }
        Ok(emb_cpu)
    }
}

impl CoreEmbedder for BgeM3Embedder {
    /// Embedding dimension (D)
    fn dim(&self) -> usize { 1024 }
    /// Maximum sequence length accepted by the model tokenizer
    fn max_len(&self) -> usize { 256 }
    /// Compute embeddings for a batch of texts on the configured device.
    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        use crate::tokenize::tokenize_batch_on_device;
        let max_len = self.max_len();
        let (input_ids, attention_mask) = tokenize_batch_on_device(&self.tokenizer, texts, max_len, &self.device, self.dtype)?;
        let token_type_ids = Tensor::zeros(attention_mask.dims(), DType::I64, &self.device)?;
        let hidden_states = self.model.forward(&input_ids, &attention_mask, &token_type_ids, None, None, None)?;
        let embedding = masked_mean_l2(&hidden_states, &attention_mask)?;
        let cpu = embedding.to_device(&Device::Cpu)?;
        let v = cpu.to_vec2::<f32>()?;
        if !v.is_empty() { assert_eq!(v[0].len(), self.dim()); }
        Ok(v)
    }
}

pub fn get_default_embedder() -> Result<Box<dyn CoreEmbedder>> {
    let use_fake = std::env::var("APP_USE_FAKE_EMBEDDINGS").ok().map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);
    if use_fake { println!("🧪 Using FakeEmbedder"); return Ok(Box::new(FakeEmbedder::new(1024))); }
    Ok(Box::new(BgeM3Embedder::new()?))
}

struct FakeEmbedder { dim: usize }
impl FakeEmbedder { fn new(dim: usize) -> Self { Self { dim } } }
impl CoreEmbedder for FakeEmbedder {
    fn dim(&self) -> usize { self.dim }
    fn max_len(&self) -> usize { 256 }
    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        use std::hash::{Hash, Hasher}; use twox_hash::XxHash64;
        let mut result = Vec::with_capacity(texts.len());
        for text in texts {
            let mut v = vec![0f32; self.dim];
            for (i, token) in text.split_whitespace().enumerate() { let mut hasher = XxHash64::with_seed(0); token.hash(&mut hasher); let h = hasher.finish(); let idx = (h as usize) % self.dim; let val = (((h >> 32) as u32) as f32) / (u32::MAX as f32); v[idx] += val + (i as f32 % 3.0) * 0.01; }
            let norm = (v.iter().map(|x| x * x).sum::<f32>()).sqrt().max(1e-6); for x in &mut v { *x /= norm; }
            result.push(v);
        }
        Ok(result)
    }
}

fn resolve_model_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("APP_MODEL_DIR") { let p = PathBuf::from(&dir); if p.exists() { println!("📦 Using APP_MODEL_DIR: {}", p.display()); return Ok(p); } }
    if let Ok(dir) = std::env::var("MODEL_DIR") { let p = PathBuf::from(&dir); if p.exists() { println!("📦 Using MODEL_DIR: {}", p.display()); return Ok(p); } }
//! localdb-embed
//!
//! Local embedding providers backed by Candle/safetensors, plus a fake
//! deterministic embedder for tests and development.
//!
//! - `BgeM3Embedder` loads XLM‑R/BGE‑M3 from `model.safetensors`
//! - `FakeEmbedder` is enabled by `APP_USE_FAKE_EMBEDDINGS=1`
//! - `get_default_embedder()` picks fake vs real at runtime
    let root = Path::new("../models/bge-m3"); if root.exists() { println!("📦 Using model dir: {}", root.display()); return Ok(root.to_path_buf()); }
    let legacy = Path::new("models/bge-m3"); if legacy.exists() { println!("📦 Using legacy model dir: {}", legacy.display()); return Ok(legacy.to_path_buf()); }
    Err(anyhow!("Could not locate BGE-M3 model directory"))
}
