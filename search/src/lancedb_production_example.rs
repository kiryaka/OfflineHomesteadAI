mod config;
use config::Config;

/// Production-ready LanceDB configuration for 100GB corpus
/// Demonstrates optimal parameters for PQ over-retrieval + flat re-ranking
fn main() -> anyhow::Result<()> {
    // Load configuration based on environment
    let config = Config::load().unwrap_or_else(|_| {
        println!("Warning: Could not load config, using defaults");
        Config::default()
    });

    // Determine environment
    let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "dev".to_string());
    let env_display = match env.as_str() {
        "prod" | "production" => "🏭 Production",
        "dev" | "development" => "🔧 Development",
        _ => "⚙️ Default",
    };

    let _data_dir = config.get_raw_txt_dir();

    println!("🚀 LanceDB Configuration for 100GB Corpus");
    println!("==========================================");
    println!("Environment: {}", env_display);
    println!();

    // Display optimized parameters
    println!("📊 Optimized LanceDB Parameters:");
    println!("  🏗️  Index Configuration (IVF_PQ):");
    println!(
        "    • num_partitions: {} (1,000-4,000 vectors per partition)",
        config.lancedb.num_partitions
    );
    println!(
        "    • num_sub_vectors: {} ({} / {} = {} sub-vectors, SIMD optimized)",
        config.lancedb.num_sub_vectors,
        config.embedding.dimension,
        config.lancedb.num_sub_vectors,
        config.embedding.dimension / config.lancedb.num_sub_vectors
    );
    println!(
        "    • metric: {} (optimal for text embeddings)",
        config.lancedb.metric
    );
    println!();

    println!("  🔍 Search Configuration (PQ + Re-ranking):");
    println!(
        "    • nprobes: {} ({}% of partitions for good recall)",
        config.lancedb_search.nprobes,
        (config.lancedb_search.nprobes * 100) / config.lancedb.num_partitions
    );
    println!(
        "    • refine_factor: {}x (over-retrieval for flat re-ranking)",
        config.lancedb_search.refine_factor
    );
    println!(
        "    • default_limit: {} (final results after re-ranking)",
        config.lancedb_search.default_limit
    );
    println!(
        "    • max_limit: {} (maximum results allowed)",
        config.lancedb_search.max_limit
    );
    println!();

    // Calculate expected performance characteristics
    let estimated_vectors = 25_000_000; // Conservative estimate for 100GB
    let vectors_per_partition = estimated_vectors / config.lancedb.num_partitions;
    let pq_compression_ratio =
        config.embedding.dimension as f32 / config.lancedb.num_sub_vectors as f32;
    let storage_reduction = (1.0 - 1.0 / pq_compression_ratio) * 100.0;

    println!("📈 Expected Performance Characteristics:");
    println!("  • Estimated vectors: {}", estimated_vectors);
    println!(
        "  • Vectors per partition: {} (optimal range: 1,000-4,000)",
        vectors_per_partition
    );
    println!("  • PQ compression ratio: {:.1}x", pq_compression_ratio);
    println!("  • Storage reduction: {:.1}%", storage_reduction);
    println!(
        "  • Search probes: {} partitions ({}% coverage)",
        config.lancedb_search.nprobes,
        (config.lancedb_search.nprobes * 100) / config.lancedb.num_partitions
    );
    println!();

    // Demonstrate search strategy
    println!("🎯 Search Strategy: PQ Over-retrieval + Flat Re-ranking");
    println!(
        "  1. PQ Search: Retrieve top {} candidates using compressed vectors",
        config.lancedb_search.default_limit * config.lancedb_search.refine_factor
    );
    println!(
        "  2. Flat Re-ranking: Compute exact distances for {} candidates",
        config.lancedb_search.default_limit * config.lancedb_search.refine_factor
    );
    println!(
        "  3. Final Results: Return top {} most relevant documents",
        config.lancedb_search.default_limit
    );
    println!();

    // Show configuration for different corpus sizes
    println!("📋 Parameter Scaling Guide:");
    println!("  Corpus Size    | Partitions | Sub-vectors | Nprobes | Refine Factor");
    println!("  --------------|------------|-------------|---------|--------------");
    println!("  10GB  (2.5M)  |    1,024   |      96     |   50    |      25");
    println!("  50GB  (12M)   |    3,072   |      96     |   150   |      35");
    println!("  100GB (25M)   |    6,144   |      96     |   300   |      40");
    println!("  500GB (125M)  |   32,768   |      96     |   1,600 |      50");
    println!();

    // Memory and storage estimates
    let vector_size_bytes = config.embedding.dimension * 4; // f32 = 4 bytes
    let pq_vector_size_bytes = config.lancedb.num_sub_vectors; // 1 byte per sub-vector
    let flat_storage_gb = (estimated_vectors * vector_size_bytes) as f64 / 1_000_000_000.0;
    let pq_storage_gb = (estimated_vectors * pq_vector_size_bytes) as f64 / 1_000_000_000.0;

    println!("💾 Storage Estimates:");
    println!("  • Flat vectors: {:.2} GB", flat_storage_gb);
    println!("  • PQ vectors: {:.2} GB", pq_storage_gb);
    println!(
        "  • Space savings: {:.1}%",
        ((flat_storage_gb - pq_storage_gb) / flat_storage_gb) * 100.0
    );
    println!();

    println!("⚡ Performance Benefits:");
    println!("  • Fast initial search with PQ vectors");
    println!("  • High accuracy through flat vector re-ranking");
    println!("  • Reduced memory footprint");
    println!("  • Scalable to very large corpora");
    println!();

    println!("🔧 Implementation Notes:");
    println!("  • Use cosine similarity for text embeddings");
    println!("  • Ensure num_sub_vectors divides dimension evenly");
    println!("  • Tune nprobes based on recall requirements");
    println!("  • Monitor refine_factor impact on latency");
    println!("  • Consider batch processing for index creation");

    Ok(())
}
