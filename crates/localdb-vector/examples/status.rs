use std::path::{Path, PathBuf};
use lancedb::query::ExecutableQuery;
use arrow_array::Array;
use futures::TryStreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ws_root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap_or(Path::new("."));
    let db_path = ws_root.join("dev_data/indexes/lancedb");
    let conn = localdb_vector::table::open_db(&db_path.to_string_lossy()).await?;
    let docs = conn.open_table("documents").execute().await?;
    let mut total = 0usize;
    let mut with_vec = 0usize;
    let mut stream = docs.query().execute().await?;
    while let Some(batch) = futures::TryStreamExt::try_next(&mut stream).await? {
        total += batch.num_rows();
        if let Some(vcol) = batch.column_by_name("vector") {
            if let Some(fsl) = vcol.as_any().downcast_ref::<arrow_array::FixedSizeListArray>() {
                for i in 0..batch.num_rows() { if fsl.is_valid(i) { with_vec += 1; } }
            }
        }
    }
    println!("documents: total={} with_vector={}", total, with_vec);
    // embeddings count
    let emb = conn.open_table("embeddings").execute().await?;
    let mut emb_rows = 0usize;
    let mut stream = emb.query().execute().await?;
    while let Some(batch) = futures::TryStreamExt::try_next(&mut stream).await? { emb_rows += batch.num_rows(); }
    println!("embeddings: rows={}", emb_rows);
    Ok(())
}
