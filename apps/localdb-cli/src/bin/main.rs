use std::env;
use std::path::PathBuf;

use localdb_core::config::Config;
use localdb_core::traits::SearchEngine;
use localdb_core::data_processor::DataProcessor;
use localdb_hybrid::HybridSearchEngine;
use localdb_text::TantivyIndexer;
use localdb_vector::LanceDbIndexer;
use localdb_embed::get_default_embedder;

fn parse_args() -> (String, Vec<String>) {
    let mut args: Vec<String> = env::args().collect();
    let prog = args.remove(0);
    if args.is_empty() { eprintln!("Usage: {} <ingest|query> [args...]", prog); std::process::exit(1); }
    let cmd = args.remove(0);
    (cmd, args)
}

fn main() -> anyhow::Result<()> {
    let config = Config::load().map_err(|e| { eprintln!("Error loading config: {}", e); e })?;
    let (cmd, args) = parse_args();
    match cmd.as_str() {
        "ingest" => {
            let data_dir = args.get(0).map(PathBuf::from).unwrap_or_else(|| {
                let dir: String = config.get("data.raw_txt_dir").unwrap_or_else(|_| "../dev_data/txt".to_string()); PathBuf::from(dir)
            });
            println!("Ingesting from {}", data_dir.display());
            let data_processor = DataProcessor::new();
            let chunks = data_processor.process_directory(&data_dir)?;
            let tantivy_index_dir: String = config.get("data.tantivy_index_dir").unwrap_or_else(|_| "../dev_data/indexes/tantivy".to_string());
            let lancedb_path = PathBuf::from(config.get::<String>("data.lancedb_index_dir").unwrap_or_else(|_| "../dev_data/indexes/lancedb".to_string()));
            let text = TantivyIndexer::new(PathBuf::from(&tantivy_index_dir))?;
            let vector = tokio::runtime::Runtime::new()?.block_on(async { LanceDbIndexer::new(&lancedb_path, "documents").await })?;
            let embedder = get_default_embedder()?;
            let engine = HybridSearchEngine::new(text, vector, embedder);
            engine.index(&chunks)?;
            println!("✅ Ingest complete ({} chunks)", chunks.len());
        }
        "query" => {
            let query_text = args.get(0).cloned().unwrap_or_else(|| {
                eprintln!("Usage: localdb-cli query \"<query>\""); std::process::exit(1)
            });
            let tantivy_index_dir: String = config.get("data.tantivy_index_dir").unwrap_or_else(|_| "../dev_data/indexes/tantivy".to_string());
            let lancedb_path = PathBuf::from(config.get::<String>("data.lancedb_index_dir").unwrap_or_else(|_| "../dev_data/indexes/lancedb".to_string()));
            let text = localdb_text::TantivySearchEngine::new(PathBuf::from(&tantivy_index_dir))?;
            let vector = tokio::runtime::Runtime::new()?.block_on(async { localdb_vector::LanceSearchEngine::new(lancedb_path, "documents").await })?;
            let embedder = get_default_embedder()?;
            // Temporary: build a minimal adapter to satisfy HybridSearchEngine generics
            struct TextAdap(localdb_text::TantivyIndexer);
            impl localdb_core::traits::TextIndexer for TextAdap {
                fn index(&self, chunks: &[localdb_core::types::DocumentChunk]) -> anyhow::Result<()> { let tmp = std::env::temp_dir().join("noop"); std::fs::create_dir_all(&tmp)?; Ok(()) }
                fn search(&self, _q: &str, _k: usize) -> anyhow::Result<Vec<localdb_core::types::SearchHit>> { Ok(vec![]) }
            }
            struct VecAdap(localdb_vector::LanceDbIndexer);
            impl localdb_core::traits::VectorIndexer for VecAdap {
                fn index(&self, _chunks: &[localdb_core::types::DocumentChunk], _embeddings: &[Vec<f32>]) -> anyhow::Result<()> { Ok(()) }
                fn search_vec(&self, _q_vec: &[f32], _k: usize) -> anyhow::Result<Vec<localdb_core::types::SearchHit>> { Ok(vec![]) }
            }
            let _ = (text, vector, embedder); // Placeholder to avoid warnings until full adapters are implemented
            println!("Query not fully wired to facade yet. Use per-engine binaries if needed.");
        }
        _ => { eprintln!("Unknown command: {}", cmd); std::process::exit(1); }
    }
    Ok(())
}
