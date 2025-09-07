use anyhow::{Result, anyhow};
use candle_core::{Device, Tensor, DType};
use tokenizers::Tokenizer;

pub fn tokenize_batch_on_device(
    tokenizer: &Tokenizer,
    texts: &[String],
    max_len: usize,
    device: &Device,
    _dtype: DType,
) -> Result<(Tensor, Tensor)> {
    // Determine pad id from tokenizer config if present; fall back to 1
    let pad_id: i64 = tokenizer.get_padding().map(|p| p.pad_id).unwrap_or(1) as i64;
    let enc = tokenizer
        .encode_batch(texts.to_vec(), true)
        .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

    let b = enc.len();
    let mut ids: Vec<i64> = Vec::with_capacity(b * max_len);
    let mut mask: Vec<i64> = Vec::with_capacity(b * max_len);

    for e in enc {
        let mut v = e.get_ids().to_vec();
        let mut m = e.get_attention_mask().to_vec();
        if v.len() > max_len { v.truncate(max_len); m.truncate(max_len); }
        if v.len() < max_len {
            let pad = max_len - v.len();
            v.extend(std::iter::repeat_n(pad_id as u32, pad));
            m.extend(std::iter::repeat_n(0u32, pad));
        }
        ids.extend(v.into_iter().map(|x| x as i64));
        mask.extend(m.into_iter().map(|x| x as i64));
    }

    let input_ids = Tensor::from_iter(ids, device)?.reshape((b, max_len))?;
    let attn_mask = Tensor::from_iter(mask, device)?.reshape((b, max_len))?;
    Ok((input_ids, attn_mask))
}

pub fn tokenize_on_device(tokenizer: &Tokenizer, text: &str, max_len: usize, device: &Device) -> Result<(Tensor, Tensor)> {
    let (ids, mask) = tokenize_batch_on_device(tokenizer, &[text.to_string()], max_len, device, DType::F32)?;
    // reshape already matches (1, max_len)
    Ok((ids, mask))
}
//! Tokenization helpers for XLM‑R/BGE‑M3.
//!
//! Provides batched tokenization on the target device/dtype. Returns input ids
//! and attention masks with shape `[B, T]`.
