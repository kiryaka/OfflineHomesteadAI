use std::path::Path;
use walkdir::WalkDir;
use twox_hash::XxHash64;
use std::hash::{Hasher, Hash};

mod config;
use config::Config;

/// Generate a deterministic facet category based on filename hash
/// Uses categories from configuration
fn deterministic_facet<'a>(filename: &str, categories: &'a [String]) -> &'a str {
    let mut hasher = XxHash64::with_seed(0);
    filename.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Use modulo to get consistent facet assignment
    let index = (hash as usize) % categories.len();
    &categories[index]
}

fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load().unwrap_or_else(|_| {
        println!("Warning: Could not load config.toml, using defaults");
        Config::default()
    });

    let data_dir = config.get_raw_txt_dir();
    
    println!("📁 Deterministic Facet Mapping");
    println!("==============================");
    println!();
    
    let mut files = Vec::new();
    for entry in WalkDir::new(&data_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && path.extension().map(|ext| ext == "txt").unwrap_or(false) {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    
    let mut facet_counts = std::collections::HashMap::new();
    
    for file_path in &files {
        let filename = file_path.file_name()
            .unwrap()
            .to_string_lossy();
        let facet = deterministic_facet(&filename, config.get_facet_categories());
        *facet_counts.entry(facet.to_string()).or_insert(0) += 1;
        
        println!("📄 {:<30} → {}", filename, facet);
    }
    
    println!();
    println!("📊 Facet Distribution:");
    for (facet, count) in facet_counts {
        println!("  {}: {} files", facet, count);
    }
    
    Ok(())
}
