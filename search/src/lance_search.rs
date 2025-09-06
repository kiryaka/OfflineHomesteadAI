use std::env;
use std::path::PathBuf;
use tantivy_demo::lance_utils::LanceSearchEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <query> [db_path] [table_name]", args[0]);
        eprintln!("Example: {} 'survival skills' ../dev_data/indexes/lancedb documents", args[0]);
        std::process::exit(1);
    }

    let query_text = &args[1];
    let db_path = args.get(2).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("../dev_data/indexes/lancedb"));
    let table_name = args.get(3).map(|s| s.as_str()).unwrap_or("documents");

    println!("🔍 LanceDB Search Only");
    println!("======================");
    println!("Query: {}", query_text);
    println!("Database path: {}", db_path.display());
    println!("Table: {}", table_name);

    // Initialize LanceDB search engine
    let search_engine = LanceSearchEngine::new(db_path, table_name).await?;

    // Show database stats
    let stats = search_engine.get_stats().await?;
    println!("📊 {}", stats);

    // Perform search
    let results = search_engine.search(query_text, 10).await?;

    println!("\n🔍 Found {} results for: \"{}\"", results.len(), query_text);

    for (i, result) in results.iter().enumerate() {
        println!("\n  {}. score={:.4}  id={}  category={}  path={}", 
                 i + 1, result.score, result.id, result.category, result.path);
        println!("     📝 Content: {}", result.content);
    }

    Ok(())
}
