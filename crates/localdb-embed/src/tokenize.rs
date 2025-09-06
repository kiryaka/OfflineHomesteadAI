use anyhow::{Result, anyhow};
use candle_core::{Device, Tensor};
use tokenizers::Tokenizer;

pub fn tokenize_on_device(tokenizer: &Tokenizer, text: &str, max_len: usize, device: &Device) -> Result<(Tensor, Tensor)> {
    let enc = tokenizer.encode(text, true).map_err(|e| anyhow!("Tokenization failed: {}", e))?;
    let mut ids = enc.get_ids().to_vec();
    let mut mask = enc.get_attention_mask().to_vec();
    if ids.len() > max_len { ids.truncate(max_len); mask.truncate(max_len); }
    if ids.len() < max_len {
        // @todo: consider tokenizer-specific pad id if available
        ids.resize(max_len, 1);
        mask.resize(max_len, 0);
    }
    let input_ids = Tensor::from_iter(ids, device)?.reshape((1, max_len))?;
    let attention_mask = Tensor::from_iter(mask, device)?.reshape((1, max_len))?;
    Ok((input_ids, attention_mask))
}
