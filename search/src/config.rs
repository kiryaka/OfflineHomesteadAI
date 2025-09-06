// Configuration module for the search system
//
// Provides a simple, flexible configuration system using figment
// for merging multiple configuration sources.

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use std::env;

/// Simple configuration system using figment for flexible config merging
pub struct Config {
    figment: Figment,
}

impl Config {
    /// Load configuration with environment-based merging
    pub fn load() -> anyhow::Result<Self> {
        let env_name = env::var("RUST_ENV").unwrap_or_else(|_| "dev".to_string());

        let figment = Figment::new()
            .merge(Toml::file("config.toml")) // Base config
            .merge(Toml::file("config.dev.toml").nested()) // Dev overrides
            .merge(Toml::file("config.prod.toml").nested()) // Prod overrides
            .merge(Env::prefixed("APP_")); // Environment variables

        let config = Self { figment };
        config.validate_for_env(&env_name)?;
        Ok(config)
    }

    /// Load specific environment config
    pub fn load_for_env(env: &str) -> anyhow::Result<Self> {
        let figment = Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Toml::file("config.dev.toml").nested())
            .merge(Toml::file("config.prod.toml").nested())
            .merge(Env::prefixed("APP_"))
            .select(env);

        let config = Self { figment };
        config.validate_for_env(env)?;
        Ok(config)
    }

    /// Get a value by key path (e.g., "data.raw_txt_dir")
    pub fn get<T>(&self, key: &str) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // Use extract with a nested key approach
        let nested_config = self.figment.clone().select(key);
        nested_config
            .extract()
            .map_err(|e| anyhow::anyhow!("Failed to get '{}': {}", key, e))
    }

    /// Extract entire config to a struct
    pub fn extract<T>(&self) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.figment
            .extract()
            .map_err(|e| anyhow::anyhow!("Failed to extract config: {}", e))
    }

    /// Validate environment-specific settings
    fn validate_for_env(&self, env: &str) -> anyhow::Result<()> {
        match env {
            "dev" | "development" => {
                let partitions: usize = self.get("lancedb.num_partitions")?;
                if partitions > 1000 {
                    return Err(anyhow::anyhow!(
                        "Dev config has too many partitions: {}. Should be <= 1000 for fast iteration", 
                        partitions
                    ));
                }

                let nprobes: usize = self.get("lancedb_search.nprobes")?;
                if nprobes > 20 {
                    return Err(anyhow::anyhow!(
                        "Dev config has too many probes: {}. Should be <= 20 for fast testing",
                        nprobes
                    ));
                }
            }
            "prod" | "production" => {
                let partitions: usize = self.get("lancedb.num_partitions")?;
                if partitions < 1000 {
                    return Err(anyhow::anyhow!(
                        "Prod config has too few partitions: {}. Should be >= 1000 for production scale", 
                        partitions
                    ));
                }

                let nprobes: usize = self.get("lancedb_search.nprobes")?;
                if nprobes < 50 {
                    return Err(anyhow::anyhow!(
                        "Prod config has too few probes: {}. Should be >= 50 for production recall",
                        nprobes
                    ));
                }
            }
            _ => {} // No validation for unknown environments
        }
        Ok(())
    }
}
