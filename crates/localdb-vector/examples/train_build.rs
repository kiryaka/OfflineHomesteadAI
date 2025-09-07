use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Minimal end-to-end: copy vectors from embeddings -> documents, compute params, build IVF_PQ index with a name, and flip active (stored in metadata is TODO)
    let ws_root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap_or(Path::new("."));
    let db_path = ws_root.join("dev_data/indexes/lancedb");
    let docs = "documents";
    let emb = "embeddings";
    let embedder_id = "local:localdb_vector::embed_provider::local::LocalProvider:d1024"; // default id shape; override as needed

    let conn = localdb_vector::table::open_db(&db_path.to_string_lossy()).await?;

    // 1) Copy vectors into serving column from embeddings side-table
    let updated = localdb_vector::index_build::sync_serving_vectors_from_embeddings(&conn, docs, emb, embedder_id).await?;
    println!("Updated serving vectors for {} rows", updated);

    // 2) Compute params
    let ready = localdb_vector::index_build::count_ready_vectors(&conn, docs).await?;
    let params = localdb_vector::index_build::compute_ivfpq_params(ready, 1024);
    println!("Training params: ready={} nlist={} m={} nbits=8", ready, params.nlist, params.m);

    // 3) Build index with a timestamped name
    let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let index_name = format!("ivfpq-{}-{}", ts, embedder_id.replace(':',"_"));
    localdb_vector::index_build::build_ivfpq_index(&conn, docs, &index_name, &params).await?;
    println!("Built index: {}", index_name);

    // 4) Minimal validation and flip
    let valid = localdb_vector::index_build::validate_index(&conn, docs, 10, 32).await?;
    if valid {
        localdb_vector::index_build::flip_active_index(&conn, docs, &index_name).await?;
        println!("Activated index: {}", index_name);
    } else {
        eprintln!("Validation failed; not flipping active index");
    }
    Ok(())
}
