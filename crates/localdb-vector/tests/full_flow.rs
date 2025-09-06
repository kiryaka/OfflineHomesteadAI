use std::collections::HashSet;
use std::path::PathBuf;
use localdb_core::data_processor::DataProcessor;
use localdb_vector::{LanceDbIndexer, LanceSearchEngine};
use tempfile::TempDir;

fn root_paths() -> (PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf();
    let data_dir = root.join("test_data/txt");
    let lancedb_dir = root.join("test_data/indexes/lancedb");
    (data_dir, lancedb_dir)
}

#[tokio::test]
async fn lancedb_full_flow() {
    std::env::set_var("APP_USE_FAKE_EMBEDDINGS", "1");
    let (data_dir, _) = root_paths();
    let processor = DataProcessor::new();
    let chunks = processor.process_directory(&data_dir).expect("process");
    let file_count: usize = { let mut ids = HashSet::new(); for c in &chunks { ids.insert(c.doc_id.clone()); } ids.len() };
    eprintln!("Lance: processed {} files into {} chunks from {}", file_count, chunks.len(), data_dir.display());
    assert!(!chunks.is_empty());
    for ch in &chunks { assert!(ch.total_chunks >= 1); assert!(ch.chunk_index < ch.total_chunks); }
    let tmp = TempDir::new().expect("tmp"); let db_path = tmp.path().to_path_buf(); let table = "documents_test_tmp";
    let indexer = LanceDbIndexer::new(&db_path, table).await.expect("indexer"); indexer.index_chunks(&chunks).await.expect("index chunks");
    eprintln!("Lance: indexed {} chunks into '{}' at {}", chunks.len(), table, db_path.display());
    let engine = LanceSearchEngine::new(db_path, table).await.expect("engine");
    let results = engine.search("fire", 5).await.expect("search");
    eprintln!("Lance: 'fire' -> {} hits", results.len()); assert!(!results.is_empty());
    if results.len() >= 2 { let s0 = results[0].score; let s1 = results[1].score; assert!(s0 >= s1); assert!((s0 - s1).abs() > 1e-6); }
    let top = &results[0]; assert!(!top.content.trim().is_empty()); assert!(top.path.contains("test_data/txt"));
}

