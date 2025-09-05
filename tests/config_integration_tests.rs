// Integration tests for configuration system
// Tests the public API and interactions between components

use tantivy_demo::config::Config;
use tantivy_demo::tests::common::ConfigTestUtils;
use anyhow::Result;

/// Integration test suite for configuration system
/// Tests environment-based loading, validation, and parameter relationships
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_config_loading() {
        let config = Config::load_dev().expect("Failed to load dev config");
        
        // Validate dev-specific optimizations
        assert!(config.lancedb.num_partitions <= 1000, 
            "Dev config should have <= 1000 partitions for fast iteration, got {}", 
            config.lancedb.num_partitions);
        
        assert!(config.lancedb_search.nprobes <= 20, 
            "Dev config should have <= 20 probes for fast testing, got {}", 
            config.lancedb_search.nprobes);
        
        assert!(config.lancedb_search.refine_factor <= 20, 
            "Dev config should have <= 20 refine factor for fast testing, got {}", 
            config.lancedb_search.refine_factor);
    }

    #[test]
    fn test_prod_config_loading() {
        let config = Config::load_prod().expect("Failed to load prod config");
        
        // Validate prod-specific optimizations
        assert!(config.lancedb.num_partitions >= 1000, 
            "Prod config should have >= 1000 partitions for production scale, got {}", 
            config.lancedb.num_partitions);
        
        assert!(config.lancedb_search.nprobes >= 50, 
            "Prod config should have >= 50 probes for production recall, got {}", 
            config.lancedb_search.nprobes);
        
        assert!(config.lancedb_search.refine_factor >= 20, 
            "Prod config should have >= 20 refine factor for production accuracy, got {}", 
            config.lancedb_search.refine_factor);
    }

    #[test]
    fn test_environment_loading() {
        // Test dev loading
        let dev_config = Config::load_for_env(Some("dev")).expect("Failed to load dev config");
        assert!(dev_config.lancedb.num_partitions <= 1000, "Dev config should be loaded correctly");
        
        // Test prod loading
        let prod_config = Config::load_for_env(Some("prod")).expect("Failed to load prod config");
        assert!(prod_config.lancedb.num_partitions >= 1000, "Prod config should be loaded correctly");
    }

    #[test]
    fn test_parameter_scaling() {
        let dev_config = Config::load_dev().expect("Failed to load dev config");
        let prod_config = Config::load_prod().expect("Failed to load prod config");
        
        // Test that prod has more partitions than dev
        assert!(prod_config.lancedb.num_partitions > dev_config.lancedb.num_partitions,
            "Prod should have more partitions than dev: {} vs {}", 
            prod_config.lancedb.num_partitions, dev_config.lancedb.num_partitions);
        
        // Test that prod has more probes than dev
        assert!(prod_config.lancedb_search.nprobes > dev_config.lancedb_search.nprobes,
            "Prod should have more probes than dev: {} vs {}", 
            prod_config.lancedb_search.nprobes, dev_config.lancedb_search.nprobes);
        
        // Test that prod has higher refine factor than dev
        assert!(prod_config.lancedb_search.refine_factor > dev_config.lancedb_search.refine_factor,
            "Prod should have higher refine factor than dev: {} vs {}", 
            prod_config.lancedb_search.refine_factor, dev_config.lancedb_search.refine_factor);
    }

    #[test]
    fn test_config_validation() {
        // Test dev validation with invalid values
        let mut invalid_dev_config = Config::default();
        invalid_dev_config.lancedb.num_partitions = 2000; // Too high for dev
        
        assert!(invalid_dev_config.validate_for_env("dev").is_err(),
            "Dev validation should fail with too many partitions");
        
        // Test prod validation with invalid values
        let mut invalid_prod_config = Config::default();
        invalid_prod_config.lancedb.num_partitions = 100; // Too low for prod
        
        assert!(invalid_prod_config.validate_for_env("prod").is_err(),
            "Prod validation should fail with too few partitions");
    }

    #[test]
    fn test_performance_characteristics() {
        let dev_config = Config::load_dev().expect("Failed to load dev config");
        let prod_config = Config::load_prod().expect("Failed to load prod config");
        
        // Calculate expected performance characteristics
        let dev_vectors_per_partition = 25000 / dev_config.lancedb.num_partitions;
        let prod_vectors_per_partition = 25000000 / prod_config.lancedb.num_partitions;
        
        // Dev should have fewer vectors per partition for faster iteration
        assert!(dev_vectors_per_partition < prod_vectors_per_partition,
            "Dev should have fewer vectors per partition for faster iteration: {} vs {}",
            dev_vectors_per_partition, prod_vectors_per_partition);
        
        // Calculate search complexity
        let dev_search_complexity = dev_config.lancedb_search.nprobes * dev_config.lancedb_search.refine_factor;
        let prod_search_complexity = prod_config.lancedb_search.nprobes * prod_config.lancedb_search.refine_factor;
        
        // Dev should have lower search complexity for faster testing
        assert!(dev_search_complexity < prod_search_complexity,
            "Dev should have lower search complexity for faster testing: {} vs {}",
            dev_search_complexity, prod_search_complexity);
    }

    #[test]
    fn test_memory_characteristics() {
        let dev_config = Config::load_dev().expect("Failed to load dev config");
        let prod_config = Config::load_prod().expect("Failed to load prod config");
        
        // Calculate memory usage estimates
        let dev_memory_per_partition = dev_config.lancedb.num_sub_vectors * 4; // 4 bytes per sub-vector
        let prod_memory_per_partition = prod_config.lancedb.num_sub_vectors * 4;
        
        // Should be the same since same embedding model
        assert_eq!(dev_memory_per_partition, prod_memory_per_partition,
            "Memory per partition should be same for same embedding model");
        
        // Calculate total memory usage
        let dev_total_memory = dev_config.lancedb.num_partitions * dev_memory_per_partition;
        let prod_total_memory = prod_config.lancedb.num_partitions * prod_memory_per_partition;
        
        // Prod should use more memory (more partitions)
        assert!(prod_total_memory > dev_total_memory,
            "Prod should use more memory than dev: {} vs {}",
            prod_total_memory, dev_total_memory);
    }
}
