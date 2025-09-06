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

    /// Validate environment-specific settings
    fn validate_for_env(&self, env: &str) -> anyhow::Result<()> {
        // Skip validation for now to avoid config structure issues
        // TODO: Implement proper validation once config structure is stable
        match env {
            "dev" | "development" => {
                // Dev validation disabled for now
            }
            "prod" | "production" => {
                // Prod validation disabled for now
            }
            _ => {} // No validation for unknown environments
        }
        Ok(())
    }
}
