use candle_core::{Device, Tensor, DType};
use localdb_embed::masked_mean_l2;

#[test]
fn masked_mean_l2_basic() {
    let dev = Device::Cpu;
    // Two tokens with hidden dim 4; second token is masked out.
    let h = Tensor::from_slice(&[1.0f32, 2.0, 3.0, 4.0,  // token 0
                                 5.0, 6.0, 7.0, 8.0],    // token 1
                               (1, 2, 4), &dev).unwrap();
    let mask = Tensor::from_slice(&[1i64, 0i64], (1, 2), &dev).unwrap()
        .to_dtype(DType::F32).unwrap();
    let out = masked_mean_l2(&h, &mask).unwrap();
    let v: Vec<Vec<f32>> = out.to_vec2().unwrap();
    let v = &v[0];
    // Mean over unmasked tokens = first token [1,2,3,4], then L2 normalize
    let norm: f32 = (1.0f32*1.0 + 2.0*2.0 + 3.0*3.0 + 4.0*4.0).sqrt();
    let expected = [1.0/norm, 2.0/norm, 3.0/norm, 4.0/norm];
    for (a,b) in v.iter().cloned().zip(expected) {
        assert!((a - b).abs() < 1e-5, "a={} b={}", a, b);
    }
}
