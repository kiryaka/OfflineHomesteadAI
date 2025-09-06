use std::{env, fs, path::PathBuf};
use localdb_core::config::Config;
use localdb_core::data_processor::DataProcessor;
use localdb_text::TantivyIndexer;
use localdb_embed::get_default_embedder;
use localdb_vector::LanceDbIndexer;

fn main() -> anyhow::Result<()> {
    let config = Config::load().map_err(|e| { eprintln!("Error loading config: {}", e); e })?;
    let args: Vec<String> = env::args().skip(1).collect();
    let mut skip_tantivy = false; let mut data_dir = None; let mut limit_lance_index = None;
    let mut i = 0; while i < args.len() { match args[i].as_str() {
        "--skip-tantivy" | "-s" => skip_tantivy = true,
        "--limit_lance_index" => { if i + 1 < args.len() { if let Ok(limit) = args[i + 1].parse::<usize>() { limit_lance_index = Some(limit); i += 1; } else { eprintln!("Error: --limit_lance_index requires a number"); std::process::exit(1); } } else { eprintln!("Error: --limit_lance_index requires a number"); std::process::exit(1); } }
        _ if !args[i].starts_with('-') => data_dir = Some(PathBuf::from(&args[i])), _ => {} } i += 1; }
    let data_dir = data_dir.unwrap_or_else(|| { let dir: String = config.get("data.raw_txt_dir").unwrap_or_else(|_| "../dev_data/txt".to_string()); PathBuf::from(dir) });
    println!("Tantivy & LanceDB Indexer\n=======================");
    println!("Data directory: {}", data_dir.display()); if skip_tantivy { println!("⚠️  Skipping Tantivy indexing (--skip-tantivy flag)"); }
    let file_count = if !skip_tantivy {
        let tantivy_index_dir: String = config.get("data.tantivy_index_dir").unwrap_or_else(|_| "../dev_data/indexes/tantivy".to_string());
        let tantivy_indexer = TantivyIndexer::new(PathBuf::from(&tantivy_index_dir))?; println!("Created Tantivy index at: {}", tantivy_index_dir);
        let count = tantivy_indexer.index_files(&data_dir)?; println!("📊 Indexed {} documents into Tantivy", count); count
    } else { 0 };
    let data_processor = DataProcessor::new();
    let chunks = if let Some(limit) = limit_lance_index { println!("🔢 Limiting LanceDB indexing to {} files", limit); data_processor.process_directory_limited(&data_dir, limit)? } else { data_processor.process_directory(&data_dir)? };
    if !chunks.is_empty() {
        let lancedb_path = PathBuf::from(config.get("data.lancedb_index_dir").unwrap_or_else(|_| "../dev_data/indexes/lancedb".to_string()));
        if lancedb_path.exists() { fs::remove_dir_all(&lancedb_path)?; }
        fs::create_dir_all(&lancedb_path)?;
        let lancedb_indexer = tokio::runtime::Runtime::new()?.block_on(async { LanceDbIndexer::new(&lancedb_path, "documents").await })?;
        let embedder = get_default_embedder()?;
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = embedder.embed_batch(&texts)?;
        tokio::runtime::Runtime::new()?.block_on(async { lancedb_indexer.index(&chunks, &embeddings).await })?;
    }
    println!("\n✅ Indexing completed successfully!"); if !skip_tantivy { println!("📊 Indexed {} documents into Tantivy", file_count); }
    println!("📊 Processed {} chunks for LanceDB", chunks.len());
    println!("\n💡 To search Tantivy, use: cargo run --bin localdb-tantivy-search '<query>'");
    println!("💡 To search LanceDB, use: cargo run --bin localdb-vector-search '<query>'");
    Ok(())
}
