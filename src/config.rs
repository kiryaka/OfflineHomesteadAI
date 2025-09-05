use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub data: DataConfig,
    pub search: SearchConfig,
    pub facets: FacetConfig,
    pub embedding: EmbeddingConfig,
    pub server: ServerConfig,
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
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_content = std::fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
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
                raw_txt_dir: "data/raw/txt".to_string(),
                tantivy_index_dir: "data/tantivy_index".to_string(),
                lancedb_index_dir: "data/lancedb_index".to_string(),
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
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
        }
    }
}
