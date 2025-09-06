use walkdir::WalkDir;
use std::path::Path;
use figment;

// Include the config module directly for this binary
include!("config.rs");

/// Generate facet category based on directory structure
/// Uses the actual directory path as the facet
fn get_facet_from_path(file_path: &Path, data_dir: &Path) -> String {
    // Get the relative path from the data directory
    let relative_path = file_path.strip_prefix(data_dir).unwrap_or(file_path);
    
    // Get the parent directory (the facet category)
    if let Some(parent) = relative_path.parent() {
        if let Some(facet) = parent.to_str() {
            return facet.to_string();
        }
    }
    
    // Fallback to "misc" if no parent directory
    "misc".to_string()
}

#[allow(dead_code)]
fn main() -> anyhow::Result<()> {
    // Load configuration without validation for facet mapping
    let figment = figment::Figment::new()
        .merge(figment::providers::Toml::file("config.toml"));
    
    let data_dir: String = figment.extract_inner("data.raw_txt_dir")?;
    let data_dir = std::path::PathBuf::from(data_dir);

    println!("📁 Directory-Based Facet Mapping");
    println!("=================================");
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
        let filename = file_path.file_name().unwrap().to_string_lossy();
        let facet = get_facet_from_path(file_path, &data_dir);
        *facet_counts.entry(facet.clone()).or_insert(0) += 1;

        println!("📄 {:<30} → {}", filename, facet);
    }

    println!();
    println!("📊 Facet Distribution:");
    for (facet, count) in facet_counts {
        println!("  {}: {} files", facet, count);
    }

    Ok(())
}
