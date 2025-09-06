use std::{
    env, fs,
    path::PathBuf,
};

mod config;
use config::Config;
mod data_processor;
use data_processor::DataProcessor;
mod embedding;
mod lancedb_indexer;
use lancedb_indexer::LanceDbIndexer;
mod tantivy_utils;
use tantivy_utils::TantivyIndexer;

fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load().map_err(|e| {
        eprintln!("Error loading config: {}", e);
        e
    })?;

    // Parse command line arguments
    let args: Vec<String> = env::args().skip(1).collect();
    
    let mut skip_tantivy = false;
    let mut data_dir = None;
    let mut limit_lance_index = None;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--skip-tantivy" | "-s" => skip_tantivy = true,
            "--limit_lance_index" => {
                if i + 1 < args.len() {
                    if let Ok(limit) = args[i + 1].parse::<usize>() {
                        limit_lance_index = Some(limit);
                        i += 1; // Skip the next argument since we consumed it
                    } else {
                        eprintln!("Error: --limit_lance_index requires a number");
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error: --limit_lance_index requires a number");
                    std::process::exit(1);
                }
            }
            _ if !args[i].starts_with('-') => data_dir = Some(PathBuf::from(&args[i])),
            _ => {}
        }
        i += 1;
    }
    
    let data_dir = data_dir.unwrap_or_else(|| {
        let dir: String = config
            .get("data.raw_txt_dir")
            .unwrap_or_else(|_| "../dev_data/txt".to_string());
        PathBuf::from(dir)
    });

    println!("Tantivy & LanceDB Indexer");
    println!("=======================");
    println!("Data directory: {}", data_dir.display());
    if skip_tantivy {
        println!("⚠️  Skipping Tantivy indexing (--skip-tantivy flag)");
    }

    // Create Tantivy index (unless skipped)
    let file_count = if !skip_tantivy {
        let tantivy_index_dir: String = config
            .get("data.tantivy_index_dir")
            .unwrap_or_else(|_| "../dev_data/indexes/tantivy".to_string());

        let tantivy_indexer = TantivyIndexer::new(PathBuf::from(&tantivy_index_dir))?;
        println!("Created Tantivy index at: {}", tantivy_index_dir);

        // Index all text files into Tantivy
        let count = tantivy_indexer.index_files(&data_dir)?;
        println!("📊 Indexed {} documents into Tantivy", count);
        count
    } else {
        0 // No files indexed
    };

    // Process files into chunks for LanceDB
    let data_processor = DataProcessor::new();
    let chunks = if let Some(limit) = limit_lance_index {
        println!("🔢 Limiting LanceDB indexing to {} files", limit);
        data_processor.process_directory_limited(&data_dir, limit)?
    } else {
        data_processor.process_directory(&data_dir)?
    };

    if !chunks.is_empty() {
        // Index chunks into LanceDB
        let lancedb_path = PathBuf::from(
            config
                .get("data.lancedb_index_dir")
                .unwrap_or_else(|_| "../dev_data/indexes/lancedb".to_string()),
        );

        // Clean up existing LanceDB index if it exists
        if lancedb_path.exists() {
            fs::remove_dir_all(&lancedb_path)?;
        }
        fs::create_dir_all(&lancedb_path)?;

        let lancedb_indexer = tokio::runtime::Runtime::new()?
            .block_on(async { LanceDbIndexer::new(&lancedb_path, "documents").await })?;

        tokio::runtime::Runtime::new()?
            .block_on(async { lancedb_indexer.index_chunks(&chunks).await })?;
    }

    println!("\n✅ Indexing completed successfully!");
    if !skip_tantivy {
        println!("📊 Indexed {} documents into Tantivy", file_count);
    }
    println!("📊 Processed {} chunks for LanceDB", chunks.len());
    println!("\n💡 To search Tantivy, use: cargo run --bin tantivy_search '<query>'");
    println!("💡 To search LanceDB, use: cargo run --bin lance_search '<query>'");

    Ok(())
}