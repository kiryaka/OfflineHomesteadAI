use anyhow::Result;
use candle_core::{DType, Tensor};

pub fn masked_mean_l2(hidden: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
    let dims = hidden.dims();
    assert_eq!(dims.len(), 3, "hidden shape must be [B,T,H]");
    let batch = dims[0];
    let _time = dims[1]; // @todo: use sequence length if needed for pooling variants
    let hidden_dim = dims[2];

    let mask = attention_mask.to_device(hidden.device())?.to_dtype(hidden.dtype())?;
    let mask_3d = mask.unsqueeze(2)?;
    let mask_broadcast = mask_3d.broadcast_as(hidden.shape()).unwrap_or(mask_3d.repeat((1, 1, hidden_dim))?);
    let masked = (hidden * &mask_broadcast)?;
    let sum = masked.sum(1)?;
    let lengths = mask.sum(1)?.unsqueeze(1)?.to_dtype(sum.dtype())?;
    let mut mean = sum.broadcast_div(&lengths)?;
    let eps_val = match hidden.dtype() { DType::F16 => 1e-6f32, _ => 1e-12f32 };
    let eps = Tensor::new(&[eps_val], hidden.device())?.to_dtype(hidden.dtype())?.unsqueeze(0)?;
    let norm = mean.sqr()?.sum_keepdim(1)?.sqrt()?;
    let norm = norm.broadcast_add(&eps)?;
    mean = mean.broadcast_div(&norm)?;
    assert_eq!(mean.dims(), &[batch, hidden_dim]);
    Ok(mean)
}
