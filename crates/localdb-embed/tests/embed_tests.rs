use localdb_embed::get_default_embedder;

#[test]
fn fake_embedder_shapes_and_determinism() {
    // Force fake embedder to avoid loading large model
    std::env::set_var("APP_USE_FAKE_EMBEDDINGS", "1");

    let embedder = get_default_embedder().expect("embedder");
    let texts = vec!["hello world".to_string(), "hello world".to_string()];
    let embs = embedder.embed_batch(&texts).expect("embed_batch");
    let v1 = &embs[0];
    let v2 = &embs[1];

    assert_eq!(v1.len(), 1024, "embedding dim is 1024");

    // Norm approximately 1.0
    let norm: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() <= 1e-3, "vector is L2-normalized (norm={norm})");

    // Deterministic for same input
    for (a, b) in v1.iter().zip(v2.iter()) { assert!((a - b).abs() <= 1e-6); }
}

