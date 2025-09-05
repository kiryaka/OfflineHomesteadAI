use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data: DataConfig,
    pub search: SearchConfig,
    pub facets: FacetConfig,
    pub embedding: EmbeddingConfig,
    pub lancedb: LanceDBConfig,
    pub lancedb_search: LanceDBSearchConfig,
    pub server: ServerConfig,
    pub dev: Option<DevConfig>,
    pub prod: Option<ProdConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    pub enable_debug_logging: bool,
    pub fast_indexing: bool,
    pub skip_expensive_operations: bool,
    pub test_data_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProdConfig {
    pub enable_debug_logging: bool,
    pub fast_indexing: bool,
    pub skip_expensive_operations: bool,
    pub monitoring_enabled: bool,
    pub performance_profiling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    pub raw_txt_dir: String,
    pub tantivy_index_dir: String,
    pub lancedb_index_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub default_limit: usize,
    pub max_limit: usize,
    pub fuzzy_max_distance: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetConfig {
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub dimension: usize,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanceDBConfig {
    pub num_partitions: usize,
    pub num_sub_vectors: usize,
    pub metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanceDBSearchConfig {
    pub nprobes: usize,
    pub refine_factor: usize,
    pub default_limit: usize,
    pub max_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        Self::load_for_env(None)
    }

    pub fn load_for_env(env: Option<&str>) -> anyhow::Result<Self> {
        let env_str = if let Some(env) = env {
            env.to_string()
        } else {
            env::var("RUST_ENV").unwrap_or_else(|_| "dev".to_string())
        };

        let config_file = match env_str.as_str() {
            "prod" | "production" => "config.prod.toml",
            "dev" | "development" => "config.dev.toml",
            _ => "config.toml", // fallback to default
        };

        let config_content = std::fs::read_to_string(config_file)
            .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", config_file, e))?;
        
        let config: Config = toml::from_str(&config_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file {}: {}", config_file, e))?;

        // Validate environment-specific settings
        config.validate_for_env(&env_str)?;
        
        Ok(config)
    }

    pub fn load_dev() -> anyhow::Result<Self> {
        Self::load_for_env(Some("dev"))
    }

    pub fn load_prod() -> anyhow::Result<Self> {
        Self::load_for_env(Some("prod"))
    }

    pub fn validate_for_env(&self, env: &str) -> anyhow::Result<()> {
        match env {
            "dev" | "development" => {
                // Dev validation
                if self.lancedb.num_partitions > 1000 {
                    return Err(anyhow::anyhow!("Dev config has too many partitions: {}. Should be <= 1000 for fast iteration", self.lancedb.num_partitions));
                }
                if self.lancedb_search.nprobes > 20 {
                    return Err(anyhow::anyhow!("Dev config has too many probes: {}. Should be <= 20 for fast testing", self.lancedb_search.nprobes));
                }
                if self.lancedb_search.refine_factor > 20 {
                    return Err(anyhow::anyhow!("Dev config has too high refine factor: {}. Should be <= 20 for fast testing", self.lancedb_search.refine_factor));
                }
            },
            "prod" | "production" => {
                // Prod validation
                if self.lancedb.num_partitions < 1000 {
                    return Err(anyhow::anyhow!("Prod config has too few partitions: {}. Should be >= 1000 for production scale", self.lancedb.num_partitions));
                }
                if self.lancedb_search.nprobes < 50 {
                    return Err(anyhow::anyhow!("Prod config has too few probes: {}. Should be >= 50 for production recall", self.lancedb_search.nprobes));
                }
            },
            _ => {} // No validation for unknown environments
        }
        Ok(())
    }

    pub fn get_raw_txt_dir(&self) -> PathBuf {
        PathBuf::from(&self.data.raw_txt_dir)
    }

    pub fn get_tantivy_index_dir(&self) -> PathBuf {
        PathBuf::from(&self.data.tantivy_index_dir)
    }

    pub fn get_lancedb_index_dir(&self) -> PathBuf {
        PathBuf::from(&self.data.lancedb_index_dir)
    }

    pub fn get_facet_categories(&self) -> &[String] {
        &self.facets.categories
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data: DataConfig {
                raw_txt_dir: "../data/raw".to_string(),
                tantivy_index_dir: "../data/indexes/tantivy".to_string(),
                lancedb_index_dir: "../data/indexes/lancedb".to_string(),
            },
            search: SearchConfig {
                default_limit: 5,
                max_limit: 100,
                fuzzy_max_distance: 4,
            },
            facets: FacetConfig {
                categories: vec![
                    "tech/math".to_string(),
                    "tech/it".to_string(),
                    "lit/fiction".to_string(),
                    "lit/romcom".to_string(),
                ],
            },
            embedding: EmbeddingConfig {
                dimension: 1536,
                model: "text-embedding-3-small".to_string(),
            },
            lancedb: LanceDBConfig {
                num_partitions: 6144,
                num_sub_vectors: 96,
                metric: "cosine".to_string(),
            },
            lancedb_search: LanceDBSearchConfig {
                nprobes: 300,
                refine_factor: 40,
                default_limit: 10,
                max_limit: 100,
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            dev: None,
            prod: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        // Test that we can load configs without panicking
        let _dev_config = Config::load_dev().expect("Failed to load dev config");
        let _prod_config = Config::load_prod().expect("Failed to load prod config");
        let _default_config = Config::load().expect("Failed to load default config");
    }

    #[test]
    fn test_config_validation_dev() {
        let config = Config::load_dev().expect("Failed to load dev config");
        
        // Test dev validation rules
        assert!(config.lancedb.num_partitions <= 1000, 
            "Dev config should have <= 1000 partitions");
        assert!(config.lancedb_search.nprobes <= 20, 
            "Dev config should have <= 20 probes");
        assert!(config.lancedb_search.refine_factor <= 20, 
            "Dev config should have <= 20 refine factor");
    }

    #[test]
    fn test_config_validation_prod() {
        let config = Config::load_prod().expect("Failed to load prod config");
        
        // Test prod validation rules
        assert!(config.lancedb.num_partitions >= 1000, 
            "Prod config should have >= 1000 partitions");
        assert!(config.lancedb_search.nprobes >= 50, 
            "Prod config should have >= 50 probes");
        assert!(config.lancedb_search.refine_factor >= 20, 
            "Prod config should have >= 20 refine factor");
    }

    #[test]
    fn test_validation_logic() {
        let mut invalid_dev_config = Config::default();
        invalid_dev_config.lancedb.num_partitions = 2000; // Too high for dev
        
        assert!(invalid_dev_config.validate_for_env("dev").is_err(),
            "Dev validation should fail with too many partitions");
        
        let mut invalid_prod_config = Config::default();
        invalid_prod_config.lancedb.num_partitions = 100; // Too low for prod
        
        assert!(invalid_prod_config.validate_for_env("prod").is_err(),
            "Prod validation should fail with too few partitions");
    }

    #[test]
    fn test_parameter_relationships() {
        let dev_config = Config::load_dev().expect("Failed to load dev config");
        let prod_config = Config::load_prod().expect("Failed to load prod config");
        
        // Prod should have more partitions than dev
        assert!(prod_config.lancedb.num_partitions > dev_config.lancedb.num_partitions);
        
        // Prod should have more probes than dev
        assert!(prod_config.lancedb_search.nprobes > dev_config.lancedb_search.nprobes);
        
        // Prod should have higher refine factor than dev
        assert!(prod_config.lancedb_search.refine_factor > dev_config.lancedb_search.refine_factor);
        
        // Both should have same sub-vectors (same embedding model)
        assert_eq!(dev_config.lancedb.num_sub_vectors, prod_config.lancedb.num_sub_vectors);
        
        // Both should have same embedding dimensions
        assert_eq!(dev_config.embedding.dimension, prod_config.embedding.dimension);
    }

    #[test]
    fn test_environment_detection() {
        // Test that environment detection works
        let dev_config = Config::load_for_env(Some("dev")).expect("Failed to load dev config");
        let prod_config = Config::load_for_env(Some("prod")).expect("Failed to load prod config");
        
        assert!(dev_config.lancedb.num_partitions < prod_config.lancedb.num_partitions);
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        
        // Test that defaults are reasonable
        assert!(config.lancedb.num_partitions > 0);
        assert!(config.lancedb.num_sub_vectors > 0);
        assert!(config.lancedb_search.nprobes > 0);
        assert!(config.lancedb_search.refine_factor > 0);
        assert!(config.embedding.dimension > 0);
        assert!(!config.embedding.model.is_empty());
    }
}
