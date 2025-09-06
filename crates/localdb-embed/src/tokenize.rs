use anyhow::{Result, anyhow};
use candle_core::{Device, Tensor, DType};
use tokenizers::Tokenizer;

pub fn tokenize_on_device(tokenizer: &Tokenizer, text: &str, max_len: usize, device: &Device) -> Result<(Tensor, Tensor)> {
    let enc = tokenizer.encode(text, true).map_err(|e| anyhow!("Tokenization failed: {}", e))?;
    let mut ids = enc.get_ids().to_vec();
    let mut mask = enc.get_attention_mask().to_vec();
    if ids.len() > max_len { ids.truncate(max_len); mask.truncate(max_len); }
    if ids.len() < max_len { let pad = max_len - ids.len(); ids.extend(std::iter::repeat(1).take(pad)); mask.extend(std::iter::repeat(0).take(pad)); }
    let input_ids = Tensor::from_iter(ids, device)?.reshape((1, max_len))?;
    let attention_mask = Tensor::from_iter(mask, device)?.reshape((1, max_len))?;
    Ok((input_ids, attention_mask))
}
