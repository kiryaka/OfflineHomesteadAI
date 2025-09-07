use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Defaults under workspace dev_data
    let ws_root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap_or(Path::new("."));
    let db_path = ws_root.join("dev_data/indexes/lancedb");
    let docs = "documents";
    let emb = "embeddings";
    let cache = "emb_cache";

    let conn = localdb_vector::table::open_db(&db_path.to_string_lossy()).await?;
    localdb_vector::table::ensure_embeddings_table(&conn, emb).await?;
    localdb_vector::table::ensure_cache_table(&conn, cache).await?;

    let provider = localdb_vector::embed_provider::local::LocalProvider::new()?;
    let n = localdb_vector::embed_backfill::backfill_embeddings(&conn, docs, emb, cache, &provider, 128, None).await?;
    println!("Backfilled {} chunks into '{}'", n, emb);
    Ok(())
}
