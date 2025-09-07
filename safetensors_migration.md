# Migration to Safetensors (Candle 0.9.1, XLM-R/BGE-M3)

This document shows how to properly switch the embedder to use safetensors with Candle 0.9.1, keeping FP16 on Metal, FP32 on CPU, and batch-safe pooling.

---

## crates/localdb-embed/src/loader.rs

```rust
use anyhow::{Result, anyhow};
use candle_core::{Device, DType, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::xlm_roberta::{Config as XlmRobertaConfig, XLMRobertaModel};
use std::{fs, path::{Path, PathBuf}};
use tokenizers::Tokenizer;

pub struct BgeM3Embedder {
    model: XLMRobertaModel,
    tok: Tokenizer,
    dev: Device,
    dt: DType,
    dim: usize,
    max_len: usize,
}

pub fn select_device() -> Device {
    match Device::new_metal(0) {
        Ok(d) => { println!("Device: Metal (MPS)"); d }
        Err(_) => { println!("Device: CPU"); Device::Cpu }
    }
}

fn select_dtype(dev: &Device) -> DType {
    match dev { Device::Metal(_) => DType::F16, _ => DType::F32 }
}

fn read_cfg(dir: &Path) -> Result<XlmRobertaConfig> {
    let p = dir.join("config.json");
    let s = fs::read_to_string(&p).map_err(|e| anyhow!("read {}: {}", p.display(), e))?;
    Ok(serde_json::from_str(&s)?)
}

fn load_tokenizer(dir: &Path) -> Result<Tokenizer> {
    let p = dir.join("tokenizer.json");
    Ok(Tokenizer::from_file(p).map_err(|e| anyhow!("tokenizer: {}", e))?)
}

fn load_model(dir: &Path, dev: &Device, dt: DType) -> Result<XLMRobertaModel> {
    let weights = dir.join("model.safetensors");
    if !weights.exists() {
        return Err(anyhow!("model.safetensors not found in {}", dir.display()));
    }
    let vb = VarBuilder::from_mmaped_safetensors(
        &[weights.to_string_lossy().as_ref()],
        candle_core::VarBuilderArgs::default()
            .with_device(dev.clone())
            .with_dtype(dt),
    )?;
    let cfg = read_cfg(dir)?;
    let model = XLMRobertaModel::new(&cfg, vb)?;
    Ok(model)
}

impl BgeM3Embedder {
    pub fn new(model_dir: PathBuf, max_len: usize) -> Result<Self> {
        let dev = select_device();
        let dt = select_dtype(&dev);
        let tok = load_tokenizer(&model_dir)?;
        let model = load_model(&model_dir, &dev, dt)?;
        Ok(Self { model, tok, dev, dt, dim: 1024, max_len })
    }

    pub fn dim(&self) -> usize { self.dim }
    pub fn max_len(&self) -> usize { self.max_len }

    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        use crate::tokenize::tokenize_batch_on_device;
        use crate::pool::masked_mean_l2;

        let (input_ids, attn_mask) = tokenize_batch_on_device(&self.tok, texts, self.max_len, &self.dev, self.dt)?;
        let shape = attn_mask.dims();
        let token_type_ids = Tensor::zeros(shape, candle_core::DType::I64, &self.dev)?;

        let hidden = self.model.forward(&input_ids, &attn_mask, &token_type_ids, None, None, None)?;
        let pooled = masked_mean_l2(&hidden, &attn_mask)?;
        let v = pooled.to_device(&Device::Cpu)?.to_vec2::<f32>()?;
        if !v.is_empty() { assert_eq!(v[0].len(), self.dim, "expected {}-d", self.dim); }
        Ok(v)
    }
}
```

---

## crates/localdb-embed/src/tokenize.rs

```rust
use anyhow::{Result, anyhow};
use candle_core::{Device, DType, Tensor};
use tokenizers::Tokenizer;

pub fn tokenize_batch_on_device(
    tok: &Tokenizer,
    texts: &[String],
    max_len: usize,
    dev: &Device,
    dt: DType,
) -> Result<(Tensor, Tensor)> {
    let pad_id = tok.get_padding().and_then(|p| p.pad_id).unwrap_or(1);
    let enc = tok.encode_batch(texts.to_vec(), true)
        .map_err(|e| anyhow!("tokenize: {}", e))?;
    let b = enc.len();
    let mut ids = Vec::with_capacity(b * max_len);
    let mut mask = Vec::with_capacity(b * max_len);

    for e in enc {
        let mut v = e.get_ids().to_vec();
        let mut m = e.get_attention_mask().to_vec();
        if v.len() > max_len { v.truncate(max_len); m.truncate(max_len); }
        if v.len() < max_len {
            let pad = max_len - v.len();
            v.extend(std::iter::repeat(pad_id).take(pad));
            m.extend(std::iter::repeat(0).take(pad));
        }
        ids.extend(v.into_iter().map(|x| x as i64));
        mask.extend(m.into_iter().map(|x| x as i64));
    }
    let input_ids = Tensor::from_iter(ids, dev)?.reshape((b, max_len))?;
    let attn_mask = Tensor::from_iter(mask, dev)?.reshape((b, max_len))?;
    Ok((input_ids, attn_mask))
}
```

---

## crates/localdb-embed/src/pool.rs

```rust
use anyhow::Result;
use candle_core::{Tensor, DType};

pub fn masked_mean_l2(h: &Tensor, attn_i64: &Tensor) -> Result<Tensor> {
    let dims = h.dims();
    let (_b, _t, hdim) = (dims[0], dims[1], dims[2]);

    let mask = attn_i64.to_device(h.device())?.to_dtype(h.dtype())?;
    let mask3 = mask.unsqueeze(2)?;
    let maskb = mask3.broadcast_as(h).unwrap_or_else(|_| mask3.repeat((1, 1, hdim)).unwrap());

    let masked = (h * &maskb)?;
    let sum = masked.sum(1)?;
    let cnt = mask.sum(1)?.unsqueeze(1)?;
    let mut emb = (&sum / &cnt)?;

    let eps = match h.dtype() {
        DType::F16 => Tensor::new(1e-6f32, h.device())?,
        _ => Tensor::new(1e-12f32, h.device())?,
    };
    let norm = (emb.sqr()?.sum_keepdim(1)?.sqrt()? + &eps)?;
    emb = (&emb / &norm)?;
    Ok(emb)
}
```

---

## crates/localdb-vector/src/writer.rs (guardrail)

```rust
for (i, v) in embeddings.iter().enumerate() {
    if v.len() != 1024 {
        return Err(anyhow::anyhow!("embedding[{}] dim {}, expected 1024", i, v.len()));
    }
}
```

---

## crates/localdb-core/src/types.rs

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub doc_id: String,
    pub path: String,
    pub content: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub category: Option<String>,
    pub meta_json: Option<String>,
}
```

---

## Test for pooling (crates/localdb-embed/tests/pooling_mask.rs)

```rust
use candle_core::{Device, Tensor, DType};
use localdb_embed::pool::masked_mean_l2;

#[test]
fn masked_mean_basic() {
    let dev = Device::Cpu;
    let h = Tensor::from_vec(vec![
        1.0f32, 3.0,   2.0, 4.0,
        10.0,   10.0,  20.0, 20.0
    ], (1,4,2), &dev).unwrap();
    let attn = Tensor::from_vec(vec![1i64,1,0,0], (1,4), &dev).unwrap();
    let out = masked_mean_l2(&h, &attn).unwrap().to_vec2::<f32>().unwrap();
    let denom = (1.5f32*1.5 + 3.5*3.5).sqrt();
    assert!((out[0][0] - 1.5/denom).abs() < 1e-3);
    assert!((out[0][1] - 3.5/denom).abs() < 1e-3);
}
```
