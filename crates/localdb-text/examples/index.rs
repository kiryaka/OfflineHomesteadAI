use std::env;
use std::path::{Path, PathBuf};

// Re-index plain text files into a Tantivy index without touching other crates.
// Usage:
//   cargo run -p localdb-text --example index -- [--dir ../dev_data/txt] [--index ../dev_data/indexes/tantivy]
// Notes:
//   - This recreates the target index directory (deletes if exists).
//   - Defaults resolve relative to the workspace root so you can run from anywhere.

fn main() -> anyhow::Result<()> {
    // Parse simple flags
    let args: Vec<String> = env::args().skip(1).collect();
    let mut data_dir: Option<PathBuf> = None;
    let mut index_dir: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => {
                if i + 1 >= args.len() { eprintln!("--dir requires a path"); std::process::exit(2); }
                data_dir = Some(PathBuf::from(&args[i + 1]));
                i += 2; continue;
            }
            "--index" => {
                if i + 1 >= args.len() { eprintln!("--index requires a path"); std::process::exit(2); }
                index_dir = Some(PathBuf::from(&args[i + 1]));
                i += 2; continue;
            }
            s if s.starts_with('-') => {
                eprintln!("Unknown flag: {}", s); std::process::exit(2);
            }
            _ => { i += 1; }
        }
    }

    // Compute workspace root from this crate's manifest dir: ../../
    let ws_root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap_or(Path::new("."));

    // Resolve paths with precedence: flag > env var > workspace defaults
    let data_dir = data_dir
        .or_else(|| env::var("TEXT_DATA_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| ws_root.join("dev_data/txt"));
    let index_dir = index_dir
        .or_else(|| env::var("TEXT_INDEX_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| ws_root.join("dev_data/indexes/tantivy"));

    println!("Tantivy re-index\n================");
    println!("Data dir : {}", data_dir.display());
    println!("Index dir: {}", index_dir.display());

    // Build a brand-new index and ingest files.
    let indexer = localdb_text::TantivyIndexer::new(index_dir)?;
    let file_count = indexer.index_files(&data_dir)?;
    println!("Done. Indexed {} files.", file_count);
    Ok(())
}

