// Library crate for tantivy_demo
// Exposes modules for use in integration tests

pub mod config;
pub mod facet_mapping;

// Test utilities module - always available for integration tests
pub mod tests {
    pub mod common {
        use crate::config::Config;
        use anyhow::Result;

        /// Test utilities for configuration testing
        pub struct ConfigTestUtils;

        impl ConfigTestUtils {
            /// Run comprehensive configuration tests
            pub fn run_all_tests() -> Result<()> {
                println!("🧪 Running Configuration Test Suite");
                println!("===================================");
                
                Self::test_dev_config()?;
                Self::test_prod_config()?;
                Self::test_config_validation()?;
                Self::test_environment_loading()?;
                Self::test_parameter_scaling()?;
                
                println!("✅ All configuration tests passed!");
                Ok(())
            }

            /// Test development configuration
            pub fn test_dev_config() -> Result<()> {
                println!("🔧 Testing Dev Configuration...");
                
                let config = Config::load_dev()?;
                
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
                
                assert!(config.search.default_limit <= 5, 
                    "Dev config should have <= 5 default limit for fast testing, got {}", 
                    config.search.default_limit);
                
                assert!(config.search.max_limit <= 50, 
                    "Dev config should have <= 50 max limit for fast testing, got {}", 
                    config.search.max_limit);
                
                // Validate dev-specific settings
                if let Some(dev_config) = &config.dev {
                    assert!(dev_config.fast_indexing, "Dev config should have fast_indexing enabled");
                    assert!(dev_config.test_data_size > 0, "Dev config should have positive test_data_size");
                }
                
                println!("  ✅ Dev configuration validated");
                Ok(())
            }

            /// Test production configuration
            pub fn test_prod_config() -> Result<()> {
                println!("🏭 Testing Prod Configuration...");
                
                let config = Config::load_prod()?;
                
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
                
                assert!(config.search.default_limit >= 5, 
                    "Prod config should have >= 5 default limit for production, got {}", 
                    config.search.default_limit);
                
                assert!(config.search.max_limit >= 50, 
                    "Prod config should have >= 50 max limit for production, got {}", 
                    config.search.max_limit);
                
                // Validate prod-specific settings
                if let Some(prod_config) = &config.prod {
                    assert!(!prod_config.fast_indexing, "Prod config should not have fast_indexing enabled");
                    assert!(prod_config.monitoring_enabled, "Prod config should have monitoring enabled");
                }
                
                println!("  ✅ Prod configuration validated");
                Ok(())
            }

            /// Test configuration validation logic
            pub fn test_config_validation() -> Result<()> {
                println!("🔍 Testing Configuration Validation...");
                
                // Test dev validation with invalid values
                let mut invalid_dev_config = Config::default();
                invalid_dev_config.lancedb.num_partitions = 2000; // Too high for dev
                
                match invalid_dev_config.validate_for_env("dev") {
                    Ok(_) => return Err(anyhow::anyhow!("Dev validation should fail with too many partitions")),
                    Err(_) => {} // Expected
                }
                
                // Test prod validation with invalid values
                let mut invalid_prod_config = Config::default();
                invalid_prod_config.lancedb.num_partitions = 100; // Too low for prod
                
                match invalid_prod_config.validate_for_env("prod") {
                    Ok(_) => return Err(anyhow::anyhow!("Prod validation should fail with too few partitions")),
                    Err(_) => {} // Expected
                }
                
                println!("  ✅ Configuration validation logic working");
                Ok(())
            }

            /// Test environment-based loading
            pub fn test_environment_loading() -> Result<()> {
                println!("🌍 Testing Environment Loading...");
                
                // Test dev loading
                let dev_config = Config::load_for_env(Some("dev"))?;
                assert!(dev_config.lancedb.num_partitions <= 1000, "Dev config should be loaded correctly");
                
                // Test prod loading
                let prod_config = Config::load_for_env(Some("prod"))?;
                assert!(prod_config.lancedb.num_partitions >= 1000, "Prod config should be loaded correctly");
                
                // Test default loading (should use RUST_ENV or default to dev)
                let default_config = Config::load()?;
                println!("  📝 Default config loaded with {} partitions", default_config.lancedb.num_partitions);
                
                println!("  ✅ Environment loading working");
                Ok(())
            }

            /// Test parameter scaling and relationships
            pub fn test_parameter_scaling() -> Result<()> {
                println!("📊 Testing Parameter Scaling...");
                
                let dev_config = Config::load_dev()?;
                let prod_config = Config::load_prod()?;
                
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
                
                // Test that sub-vectors are consistent (same embedding model)
                assert_eq!(dev_config.lancedb.num_sub_vectors, prod_config.lancedb.num_sub_vectors,
                    "Dev and prod should have same sub-vectors for same embedding model");
                
                // Test that embedding dimensions are consistent
                assert_eq!(dev_config.embedding.dimension, prod_config.embedding.dimension,
                    "Dev and prod should have same embedding dimensions");
                
                println!("  ✅ Parameter scaling validated");
                Ok(())
            }

            /// Performance regression test
            pub fn test_performance_regression() -> Result<()> {
                println!("⚡ Testing Performance Regression...");
                
                let dev_config = Config::load_dev()?;
                let prod_config = Config::load_prod()?;
                
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
                
                println!("  ✅ Performance regression test passed");
                Ok(())
            }

            /// Memory usage regression test
            pub fn test_memory_regression() -> Result<()> {
                println!("💾 Testing Memory Regression...");
                
                let dev_config = Config::load_dev()?;
                let prod_config = Config::load_prod()?;
                
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
                
                println!("  ✅ Memory regression test passed");
                Ok(())
            }
        }
    }
}

