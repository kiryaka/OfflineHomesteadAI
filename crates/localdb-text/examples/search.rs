use std::env;
use std::path::{Path, PathBuf};

// Query an existing Tantivy index and print results.
// Usage:
//   cargo run -p localdb-text --example search -- "your query" \
//     [--index ../dev_data/indexes/tantivy] [--limit 10] [--facets]

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: cargo run -p localdb-text --example search -- <query> [--index DIR] [--limit N] [--facets]");
        std::process::exit(1);
    }
    let mut query = String::new();
    let mut index_dir: Option<PathBuf> = None;
    let mut limit: usize = 10;
    let mut show_facets = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--index" => {
                if i + 1 >= args.len() { eprintln!("--index requires a path"); std::process::exit(2); }
                index_dir = Some(PathBuf::from(&args[i + 1]));
                i += 2; continue;
            }
            "--limit" => {
                if i + 1 >= args.len() { eprintln!("--limit requires a number"); std::process::exit(2); }
                limit = args[i + 1].parse().unwrap_or(limit);
                i += 2; continue;
            }
            "--facets" => { show_facets = true; i += 1; continue; }
            s if s.starts_with("-") => {
                eprintln!("Unknown flag: {}", s); std::process::exit(2);
            }
            s => {
                if query.is_empty() { query = s.to_string(); }
                i += 1; continue;
            }
        }
    }

    if query.is_empty() {
        eprintln!("Missing <query> argument");
        std::process::exit(1);
    }

    // Resolve index path precedence: flag > TEXT_INDEX_DIR > workspace-relative fallback
    let index_dir = if let Some(id) = index_dir {
        id
    } else if let Ok(env_path) = env::var("TEXT_INDEX_DIR") {
        PathBuf::from(env_path)
    } else {
        // Compute workspace root from this crate's manifest dir: ../../
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors().nth(2)
            .unwrap_or(Path::new("."));
        base.join("dev_data/indexes/tantivy")
    };

    println!("Tantivy search\n==============");
    println!("Index: {}", index_dir.display());
    println!("Query: {} (limit {})\n", query, limit);

    let engine = localdb_text::TantivySearchEngine::new(index_dir)?;
    let hits = engine.search(&query, limit)?;
    for (i, h) in hits.iter().enumerate() {
        println!("{:>2}. score={:.3} id={} path={} category={}\n    snippet: {}",
            i + 1, h.score, h.id, h.path, h.category, h.snippet);
    }

    if show_facets {
        println!("\nFacets:");
        for (facet, count) in engine.get_facet_counts(&query)? {
            println!("  {} -> {}", facet, count);
        }
    }

    Ok(())
}
