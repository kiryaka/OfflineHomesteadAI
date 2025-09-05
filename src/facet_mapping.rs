use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use twox_hash::XxHash64;
use std::hash::{Hasher, Hash};

/// Generate a deterministic facet category based on filename hash
/// 2-level tree: tech/math, tech/it, lit/fiction, lit/romcom
fn deterministic_facet(filename: &str) -> &'static str {
    let mut hasher = XxHash64::with_seed(0);
    filename.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Use modulo 4 to get consistent facet assignment
    match hash % 4 {
        0 => "tech/math",
        1 => "tech/it", 
        2 => "lit/fiction",
        _ => "lit/romcom",
    }
}

fn main() -> anyhow::Result<()> {
    let data_dir = Path::new("./data");
    
    println!("📁 Deterministic Facet Mapping");
    println!("==============================");
    println!();
    
    let mut files = Vec::new();
    for entry in WalkDir::new(data_dir).into_iter().filter_map(Result::ok) {
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
        let facet = deterministic_facet(&filename);
        *facet_counts.entry(facet).or_insert(0) += 1;
        
        println!("📄 {:<30} → {}", filename, facet);
    }
    
    println!();
    println!("📊 Facet Distribution:");
    for (facet, count) in facet_counts {
        println!("  {}: {} files", facet, count);
    }
    
    Ok(())
}
