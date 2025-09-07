use std::env;
use std::path::PathBuf;
use localdb_vector::LanceSearchEngine;
use localdb_embed::get_default_embedder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <query> [--limit N] [db_path] [table_name]", args[0]);
        eprintln!("Example: {} 'survival skills' --limit 5 ../dev_data/indexes/lancedb documents", args[0]);
        std::process::exit(1);
    }
    let query_text = &args[1];
    let mut limit = 10usize;
    let mut db_path = PathBuf::from("../dev_data/indexes/lancedb");
    let mut table_name = "documents";
    let mut i = 2; while i < args.len() { match args[i].as_str() {
        "--limit" => { if i + 1 < args.len() { if let Ok(l) = args[i + 1].parse::<usize>() { limit = l; i += 1; } else { eprintln!("Error: --limit requires a number"); std::process::exit(1); } } else { eprintln!("Error: --limit requires a number"); std::process::exit(1); } }
        _ if !args[i].starts_with('-') => { if db_path == PathBuf::from("../dev_data/indexes/lancedb") { db_path = PathBuf::from(&args[i]); } else { table_name = &args[i]; } }
        _ => {} } i += 1; }
    println!("🔍 localdb-vector-search\n======================");
    println!("Query: {}", query_text); println!("Database path: {}", db_path.display()); println!("Table: {}", table_name);
    let embedder = get_default_embedder()?;
    let search_engine = LanceSearchEngine::new(db_path, table_name, embedder).await?;
    let results = search_engine.search(query_text, limit).await?;
    println!("\n🔍 Found {} results for: \"{}\"", results.len(), query_text);
    for (i, result) in results.iter().enumerate() {
        println!("\n  {}. score={:.4}  id={}  category={}  path={}", i + 1, result.score, result.id, result.category, result.path);
        println!("     📝 Content: {}", result.content);
    }
    Ok(())
}
