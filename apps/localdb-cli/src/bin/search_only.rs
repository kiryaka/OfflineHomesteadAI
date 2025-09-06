use std::env;
use std::path::PathBuf;
use localdb_text::TantivySearchEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <query> [index_dir]", args[0]);
        eprintln!("Example: {} 'survival AND fire' ../dev_data/indexes/tantivy", args[0]);
        std::process::exit(1);
    }
    let query_text = &args[1];
    let index_dir = args.get(2).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("../dev_data/indexes/tantivy"));
    println!("🔍 localdb-search-only\n==================");
    println!("Query: {}", query_text); println!("Index directory: {}", index_dir.display());
    let search_engine = TantivySearchEngine::new(index_dir)?;
    let results = search_engine.search(query_text, 10)?;
    println!("\n🔍 Found {} results for: \"{}\"", results.len(), query_text);
    for (i, result) in results.iter().enumerate() {
        println!("\n  {}. score={:.4}  id={}  category={}  path={}", i + 1, result.score, result.id, result.category, result.path);
        println!("     📝 Context: {}", result.snippet);
    }
    println!("\n📊 Facet counts:");
    let facets = search_engine.get_facet_counts(query_text)?;
    for (facet, count) in facets { println!("  {}: {} documents", facet, count); }
    Ok(())
}

