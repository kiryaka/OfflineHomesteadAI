use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::xlm_roberta::{XLMRobertaModel, Config as XLMRobertaConfig};
use tokenizers::Tokenizer;

pub trait Embedder: Send + Sync { fn embed_text(&self, text: &str) -> Result<Vec<f32>>; }

pub struct EmbeddingModel { model: XLMRobertaModel, tokenizer: Tokenizer, device: Device }

impl EmbeddingModel {
    pub fn new() -> Result<Self> {
        let device = Device::new_metal(0).map_err(|e| anyhow!("Failed to initialize Metal device: {}", e))?;
        println!("🚀 Device: Metal (MPS)");
        println!("🔄 Loading BGE-M3 model from local files...");
        let model_dir = resolve_model_dir()?;
        println!("📥 Loading tokenizer...");
        let tokenizer_path = model_dir.join("tokenizer.json");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer from {}: {}", tokenizer_path.display(), e))?;
        println!("📥 Loading model config...");
        let config_path = model_dir.join("config.json");
        let config: XLMRobertaConfig = serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;
        println!("📥 Loading model weights...");
        let weights_path = model_dir.join("pytorch_model.bin");
        let dtype = DType::F32;
        let weights = candle_core::pickle::read_all(&weights_path)?;
        let weights_map: std::collections::HashMap<String, candle_core::Tensor> = weights.into_iter().collect();
        let vb = VarBuilder::from_tensors(weights_map, dtype, &device);
        println!("🔧 Loading model...");
        let model = XLMRobertaModel::new(&config, vb)?;
        println!("✅ BGE-M3 model loaded successfully!");
        Ok(Self { model, tokenizer, device })
    }

    pub fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let start = Instant::now();
        let max_len = 256usize;
        let enc = self.tokenizer.encode(text, true).map_err(|e| anyhow!("Tokenization failed: {}", e))?;
        let mut ids = enc.get_ids().to_vec();
        let mut mask = enc.get_attention_mask().to_vec();
        if ids.len() > max_len { ids.truncate(max_len); mask.truncate(max_len); }
        if ids.len() < max_len { let pad = max_len - ids.len(); ids.extend(std::iter::repeat(1).take(pad)); mask.extend(std::iter::repeat(0).take(pad)); }
        let input_ids = Tensor::from_iter(ids, &self.device)?.reshape((1, max_len))?;
        let attention_mask = Tensor::from_iter(mask, &self.device)?.reshape((1, max_len))?;
        let token_type_ids = Tensor::zeros((1, max_len), DType::I64, &self.device)?;
        let hidden_states = self.model.forward(&input_ids, &attention_mask, &token_type_ids, None, None, None)?;
        let hidden_shape = hidden_states.shape(); let hidden_dims = hidden_shape.dims();
        assert_eq!(hidden_dims.len(), 3); assert_eq!(hidden_dims[0], 1); assert_eq!(hidden_dims[1], max_len);
        let h = hidden_states; let attn = attention_mask;
        let (_b, _t, hdim) = { let d = h.dims(); (d[0], d[1], d[2]) };
        let mask = attn.to_device(h.device())?.to_dtype(h.dtype())?;
        let mask_3d = mask.unsqueeze(2)?; let mask_b = mask_3d.broadcast_as(h.shape()).unwrap_or(mask_3d.repeat((1,1,hdim))?);
        let masked = (&h * &mask_b)?; let sum = masked.sum(1)?; let lens = mask.sum(1)?.unsqueeze(1)?;
        let lens = lens.to_dtype(sum.dtype())?; let mut emb = sum.broadcast_div(&lens)?;
        let eps_val = match h.dtype() { candle_core::DType::F16 => 1e-6f32, _ => 1e-12f32 };
        let eps = Tensor::new(&[eps_val], h.device())?.to_dtype(h.dtype())?.unsqueeze(0)?;
        let norm = emb.sqr()?.sum_keepdim(1)?.sqrt()?; let norm = norm.broadcast_add(&eps)?; emb = emb.broadcast_div(&norm)?;
        let emb_cpu = emb.to_device(&Device::Cpu)?.squeeze(0)?.to_vec1()?; assert_eq!(emb_cpu.len(), 1024);
        if start.elapsed().as_millis() > 100 { println!("⚠️  Slow embedding"); }
        Ok(emb_cpu)
    }
}

impl Embedder for EmbeddingModel { fn embed_text(&self, text: &str) -> Result<Vec<f32>> { self.embed_text(text) } }

struct FakeEmbedder { dim: usize }
impl FakeEmbedder { fn new(dim: usize) -> Self { Self { dim } } }
impl Embedder for FakeEmbedder {
    fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        use std::hash::{Hash, Hasher}; use twox_hash::XxHash64;
        let mut v = vec![0f32; self.dim];
        for (i, token) in text.split_whitespace().enumerate() { let mut hasher = XxHash64::with_seed(0); token.hash(&mut hasher); let h = hasher.finish(); let idx = (h as usize) % self.dim; let val = (((h >> 32) as u32) as f32) / (u32::MAX as f32); v[idx] += val + (i as f32 % 3.0) * 0.01; }
        let norm = (v.iter().map(|x| x * x).sum::<f32>()).sqrt().max(1e-6); for x in &mut v { *x /= norm; } Ok(v)
    }
}

pub fn get_default_embedder() -> Result<Box<dyn Embedder>> {
    let use_fake = std::env::var("APP_USE_FAKE_EMBEDDINGS").ok().map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);
    if use_fake { println!("🧪 Using FakeEmbedder"); return Ok(Box::new(FakeEmbedder::new(1024))); }
    Ok(Box::new(EmbeddingModel::new()?))
}

fn resolve_model_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("APP_MODEL_DIR") { let p = PathBuf::from(&dir); if p.exists() { println!("📦 Using APP_MODEL_DIR: {}", p.display()); return Ok(p); } }
    if let Ok(dir) = std::env::var("MODEL_DIR") { let p = PathBuf::from(&dir); if p.exists() { println!("📦 Using MODEL_DIR: {}", p.display()); return Ok(p); } }
    let root = Path::new("../models/bge-m3"); if root.exists() { println!("📦 Using model dir: {}", root.display()); return Ok(root.to_path_buf()); }
    let legacy = Path::new("models/bge-m3"); if legacy.exists() { println!("📦 Using legacy model dir: {}", legacy.display()); return Ok(legacy.to_path_buf()); }
    Err(anyhow!("Could not locate BGE-M3 model directory"))
}

