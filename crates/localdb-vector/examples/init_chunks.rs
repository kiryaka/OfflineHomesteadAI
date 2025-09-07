use std::path::{Path, PathBuf};
use localdb_vector::writer::LanceDocument;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Ingest raw txt files into documents table with vector=null and embedding_status=new
    // Resolve workspace root from this crate: ../../
    let ws_root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap_or(Path::new("."));
    let data_dir = ws_root.join("dev_data/txt");
    let db_path = ws_root.join("dev_data/indexes/lancedb");
    let table = "documents";

    let conn = localdb_vector::table::open_db(&db_path.to_string_lossy()).await?;
    // Ensure table exists; writer will create if missing on first insert
    let processor = localdb_core::data_processor::DataProcessor::new();
    let chunks = processor.process_directory(&data_dir)?;
    println!("Preparing {} chunks...", chunks.len());
    // Convert to docs with no vectors
    let docs: Vec<LanceDocument> = chunks.iter().map(|c| LanceDocument{
        id: c.id.clone(), doc_id: c.doc_id.clone(), doc_path: c.doc_path.clone(),
        category: c.category.clone(), category_text: c.category_text.clone(), content: c.content.clone(),
        chunk_index: c.chunk_index, total_chunks: c.total_chunks, vector: Vec::new()
    }).collect();

    let indexer = localdb_vector::LanceDbIndexer::new(&db_path, table).await?;
    // Use internal helper via public API: index requires embeddings; we'll insert batches with empty vectors by calling private conversion path.
    // For simplicity, reusing index with zero embeddings will mark rows as 'new'.
    // Build a zero-vecs slice matching docs
    let empty: Vec<Vec<f32>> = vec![Vec::new(); docs.len()];
    indexer.index(&chunks, &empty).await?;
    println!("Initialized documents table with {} chunks", chunks.len());
    Ok(())
}
