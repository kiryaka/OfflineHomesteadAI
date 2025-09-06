use std::path::PathBuf;
use localdb_text::{TantivyIndexer, TantivySearchEngine};
use walkdir::WalkDir;

fn root_paths() -> (PathBuf, PathBuf) {
    // crates/localdb-text -> crates -> repo root
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf();
    let data_dir = root.join("test_data/txt");
    let index_dir = root.join("test_data/indexes/tantivy");
    (data_dir, index_dir)
}

#[test]
fn tantivy_full_flow() {
    let (data_dir, index_dir) = root_paths();
    let expected_txt: usize = WalkDir::new(&data_dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()).filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("txt")).count();
    eprintln!("Tantivy: indexing {} .txt files from {} → {}", expected_txt, data_dir.display(), index_dir.display());
    let indexer = TantivyIndexer::new(index_dir.clone()).expect("indexer");
    let count = indexer.index_files(&data_dir).expect("index files");
    eprintln!("Tantivy: indexed {} documents", count);
    assert_eq!(count, expected_txt);

    let engine = TantivySearchEngine::new(index_dir).expect("engine");
    for q in ["firecraft", "computer", "networking"] { let results = engine.search(q, 10).expect("search"); eprintln!("q='{}' -> {} hits", q, results.len()); if results.len() >= 2 { let s0 = results[0].score; let s1 = results[1].score; assert!(s0 >= s1); assert!((s0 - s1).abs() > 1e-6); } }
    let facets = TantivySearchEngine::new(root_paths().1).expect("engine").get_facet_counts("fire").expect("facets");
    assert!(!facets.is_empty());
}

