use anyhow::{Result, anyhow};
use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::xlm_roberta::{XLMRobertaModel, Config as XLMRobertaConfig};
use tokenizers::Tokenizer;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Real BGE-M3 embedding model (uses XLM-RoBERTa backbone)
pub struct EmbeddingModel {
    model: XLMRobertaModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl EmbeddingModel {
    pub fn new() -> Result<Self> {
        // Force Metal - no fallback to CPU
        let device = Device::new_metal(0)
            .map_err(|e| anyhow!("Failed to initialize Metal device: {}. Make sure you're on macOS with Metal support.", e))?;
        println!("🚀 Device: Metal (MPS) - Fast GPU acceleration enabled!");
        
        println!("🔄 Loading BGE-M3 model from local files...");

        // Resolve model directory (supports root-level ../models and legacy ./models)
        let model_dir = resolve_model_dir()?;
        
        // Load tokenizer
        println!("📥 Loading tokenizer...");
        let tokenizer_path = model_dir.join("tokenizer.json");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer from {}: {}", tokenizer_path.display(), e))?;
        
        // Load model config (BGE-M3 uses XLM-RoBERTa config)
        println!("📥 Loading model config...");
        let config_path = model_dir.join("config.json");
        let config: XLMRobertaConfig = serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;
        
        // Load model weights with proper device and dtype
        println!("📥 Loading model weights...");
        let weights_path = model_dir.join("pytorch_model.bin");
        
        // Use f32 for simplicity (Metal will still be fast)
        let dtype = DType::F32;
        
        let weights = candle_core::pickle::read_all(&weights_path)?;
        let weights_map: std::collections::HashMap<String, candle_core::Tensor> = weights.into_iter().collect();
        let vb = VarBuilder::from_tensors(weights_map, dtype, &device);
        
        // Create model
        println!("🔧 Loading model...");
        let model = XLMRobertaModel::new(&config, vb)?;
        
        println!("✅ BGE-M3 model loaded successfully!");
        
        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    pub fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let start = Instant::now();
        
        // Tokenize with padding/truncation
        let max_len = 256usize;
        let enc = self.tokenizer
            .encode(text, true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;
        
        let mut ids = enc.get_ids().to_vec();
        let mut mask = enc.get_attention_mask().to_vec();
        
        // Truncate if too long
        if ids.len() > max_len { 
            ids.truncate(max_len); 
            mask.truncate(max_len); 
        }
        
        // Pad if too short
        if ids.len() < max_len {
            let pad = max_len - ids.len();
            ids.extend(std::iter::repeat(1).take(pad)); // 1 is <pad> token
            mask.extend(std::iter::repeat(0).take(pad));
        }
        
        // Convert to tensors
        let input_ids = Tensor::from_iter(ids, &self.device)?
            .reshape((1, max_len))?; // [1, max_len]
        let attention_mask = Tensor::from_iter(mask, &self.device)?
            .reshape((1, max_len))?; // [1, max_len]
        
        // Run model inference
        // XLM-RoBERTa doesn't use token_type_ids, so we pass zeros
        let token_type_ids = Tensor::zeros((1, max_len), DType::I64, &self.device)?;
        let outputs = self.model.forward(&input_ids, &attention_mask, &token_type_ids, None, None, None)?;
        let hidden_states = outputs; // [1, max_len, hidden_size]
        
        // Assert expected dimensions
        let hidden_shape = hidden_states.shape();
        let hidden_dims = hidden_shape.dims();
        assert_eq!(hidden_dims.len(), 3, "Expected 3D tensor for hidden states");
        assert_eq!(hidden_dims[0], 1, "Expected batch size 1");
        assert_eq!(hidden_dims[1], max_len, "Expected sequence length {}", max_len);
        let hidden_size = hidden_dims[2];
        assert_eq!(hidden_size, 1024, "Expected hidden size 1024, got {}", hidden_size);
        
        // h: [B, T, H] last_hidden_state from XLMRobertaModel
        // attn: [B, T] attention mask with 1 for real tokens, 0 for pads
        let h = hidden_states; // [1, max_len, 1024]
        let attn = attention_mask; // [1, max_len]

        let (_b, _t, hdim) = {
            let d = h.dims();
            (d[0], d[1], d[2])
        };

        // 1) Make sure mask is on the same device and dtype as `h`
        let mask = attn
            .to_device(h.device())?
            .to_dtype(h.dtype())?;                // match f16/f32 with hidden states

        // 2) Unsqueeze to [B, T, 1]
        let mask_3d = mask.unsqueeze(2)?;         // [B, T, 1]

        // 3) Explicitly broadcast to [B, T, H]
        let mask_b = match mask_3d.broadcast_as(h.shape()) {
            Ok(m) => m,
            Err(_) => {
                // Fallback: repeat along the last dim
                mask_3d.repeat((1, 1, hdim))?     // -> [B, T, H]
            }
        };

        // 4) Elementwise mask and mean-pool over T
        let masked = (&h * &mask_b)?;             // [B, T, H]
        let sum = masked.sum(1)?;                  // [B, H]
        let lens = mask.sum(1)?.unsqueeze(1)?;     // [B, 1]

        // Avoid dtype surprises in the divide
        let lens = lens.to_dtype(sum.dtype())?;
        let mut emb = sum.broadcast_div(&lens)?;   // [B, H]

        // 5) L2-normalize (epsilon in same dtype/device)
        let eps_val = match h.dtype() {
            candle_core::DType::F16 => 1e-6f32,
            _ => 1e-12f32,
        };
        let eps = Tensor::new(&[eps_val], h.device())?.to_dtype(h.dtype())?.unsqueeze(0)?; // [1]
        let norm = emb.sqr()?.sum_keepdim(1)?.sqrt()?; // [1, 1]
        let norm = norm.broadcast_add(&eps)?; // [1, 1]
        emb = emb.broadcast_div(&norm)?; // [1, 1024]
        
        // Convert to Vec<f32> and remove batch dimension
        let emb_cpu = emb.to_device(&Device::Cpu)?.squeeze(0)?.to_vec1()?;
        
        // Final dimension assertion
        assert_eq!(emb_cpu.len(), 1024, "Expected embedding dimension 1024, got {}", emb_cpu.len());
        
        let duration = start.elapsed();
        if duration.as_millis() > 100 {
            println!("⚠️  Slow embedding: {:?} for text length {}", duration, text.len());
        }
        
        Ok(emb_cpu)
    }
}

fn resolve_model_dir() -> Result<PathBuf> {
    // Allow explicit override via env var
    if let Ok(dir) = std::env::var("APP_MODEL_DIR") {
        let p = PathBuf::from(&dir);
        if p.exists() {
            println!("📦 Using APP_MODEL_DIR: {}", p.display());
            return Ok(p);
        }
    }
    if let Ok(dir) = std::env::var("MODEL_DIR") {
        let p = PathBuf::from(&dir);
        if p.exists() {
            println!("📦 Using MODEL_DIR: {}", p.display());
            return Ok(p);
        }
    }

    // Preferred root-level location after move
    let root = Path::new("../models/bge-m3");
    if root.exists() {
        println!("📦 Using model dir: {}", root.display());
        return Ok(root.to_path_buf());
    }

    // Legacy in-crate location (fallback)
    let legacy = Path::new("models/bge-m3");
    if legacy.exists() {
        println!("📦 Using legacy model dir: {}", legacy.display());
        return Ok(legacy.to_path_buf());
    }

    Err(anyhow!(
        "Could not locate BGE-M3 model directory. Checked APP_MODEL_DIR, MODEL_DIR, ../models/bge-m3, and models/bge-m3"
    ))
}
