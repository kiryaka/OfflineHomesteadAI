use std::fs;
use std::io::Write;
use tempfile::TempDir;

use localdb_core::data_processor::DataProcessor;

#[test]
fn process_directory_single_small_file() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    let file_path = dir.join("a.txt");
    let mut f = fs::File::create(&file_path).unwrap();
    writeln!(f, "Short text").unwrap();

    let processor = DataProcessor::new();
    let chunks = processor.process_directory(dir).expect("process");

    assert_eq!(chunks.len(), 1, "one small paragraph becomes one chunk");
    assert_eq!(chunks[0].content.trim(), "Short text");
}

#[test]
fn process_directory_limited_two_files_limit_one() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path();
    fs::write(dir.join("a.txt"), "alpha bravo").unwrap();
    fs::write(dir.join("b.txt"), "charlie delta").unwrap();

    let processor = DataProcessor::new();
    let chunks = processor
        .process_directory_limited(dir, 1)
        .expect("process limited");

    // Only chunks from one document should be present
    let mut doc_ids = std::collections::HashSet::new();
    for c in &chunks { doc_ids.insert(c.doc_id.clone()); }
    assert_eq!(doc_ids.len(), 1, "limited to one source document");
}

