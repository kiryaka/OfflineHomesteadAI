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
    
    let index = (hash as usize) % categories.len();
    &categories[index]
}

/// Generate a simple embedding vector based on filename hash
/// This is a placeholder - in real implementation, you'd use actual embeddings
fn generate_embedding(filename: &str, content: &str) -> Vec<f32> {
    let mut hasher = XxHash64::with_seed(0);
    filename.hash(&mut hasher);
    content.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Generate deterministic 128-dimensional vector
    let mut rng = std::collections::hash_map::DefaultHasher::new();
    rng.write_u64(hash);
    let mut vector = vec![0f32; 128];
    
    for i in 0..128 {
        rng.write_u64(hash.wrapping_add(i as u64));
        vector[i] = (rng.finish() % 1000) as f32 / 1000.0;
    }
    
    vector
}

fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load().unwrap_or_else(|_| {
        println!("Warning: Could not load config.toml, using defaults");
        Config::default()
    });

    let data_dir = config.get_raw_txt_dir();
    
    println!("🚀 LanceDB + Tantivy Hybrid Search Demo");
    println!("=======================================");
    println!();
    
    let mut files = Vec::new();
    for entry in WalkDir::new(&data_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && path.extension().map(|ext| ext == "txt").unwrap_or(false) {
            files.push(path.to_path_buf());
        }
    }
    files.sort();
    
    println!("📊 Data Preparation for Hybrid Search:");
    println!();
    
    let mut facet_counts = std::collections::HashMap::new();
    
    for (i, file_path) in files.iter().enumerate() {
        let filename = file_path.file_name()
            .unwrap()
            .to_string_lossy();
        let facet = deterministic_facet(&filename, config.get_facet_categories());
        *facet_counts.entry(facet.to_string()).or_insert(0) += 1;
        
        // Read content (simplified)
        let content = std::fs::read_to_string(file_path)
            .unwrap_or_else(|_| "Content unavailable".to_string());
        
        // Generate embedding
        let embedding = generate_embedding(&filename, &content);
        
        println!("📄 {:<30} | {} | embedding: [{:.3}, {:.3}, ...]", 
                filename, facet, embedding[0], embedding[1]);
        
        if i >= 4 {  // Show only first 5 files
            println!("... and {} more files", files.len() - 5);
            break;
        }
    }
    
    println!();
    println!("📊 Facet Distribution (same as Tantivy):");
    for (facet, count) in facet_counts {
        println!("  {}: {} files", facet, count);
    }
    
    println!();
    println!("💡 Hybrid Search Architecture:");
    println!("  🔍 Tantivy: Exact text search + faceting");
    println!("  🧠 LanceDB: Vector similarity search");
    println!("  🔗 Merge: Combine results by document ID");
    println!();
    println!("Example search flow:");
    println!("  1. User queries: 'coffee house'");
    println!("  2. Tantivy finds: docs with 'coffee' AND 'house'");
    println!("  3. LanceDB finds: docs similar to 'coffee house' embedding");
    println!("  4. Merge results: deduplicate and re-rank");
    
    Ok(())
}
