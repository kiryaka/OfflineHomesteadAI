use localdb_embed::{BgeM3Embedder};

fn main() -> anyhow::Result<()> {
    let embedder = BgeM3Embedder::new()?;
    let texts = vec!["hello world".to_string(), "rust embeddings".to_string()];
    let embs = <dyn localdb_core::traits::Embedder>::embed_batch(&embedder, &texts)?;
    println!("B={} dim={}", embs.len(), embedder.dim());
    Ok(())
}
