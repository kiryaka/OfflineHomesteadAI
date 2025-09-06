use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use std::env;

pub struct Config {
    figment: Figment,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let env_name = env::var("RUST_ENV").unwrap_or_else(|_| "dev".to_string());

        let mut figment = Figment::new().merge(Toml::file("config.toml"));
        match env_name.as_str() {
            "dev" | "development" => figment = figment.merge(Toml::file("config.dev.toml")),
            "prod" | "production" => figment = figment.merge(Toml::file("config.prod.toml")),
            "test" | "testing" => figment = figment.merge(Toml::file("config.test.toml")),
            _ => {}
        }
        figment = figment.merge(Env::prefixed("APP_"));

        let config = Self { figment };
        config.validate_for_env(&env_name)?;
        Ok(config)
    }

    pub fn get<T>(&self, key: &str) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.figment
            .extract_inner(key)
            .map_err(|e| anyhow::anyhow!("Failed to get '{}': {}", key, e))
    }

    fn validate_for_env(&self, env: &str) -> anyhow::Result<()> {
        match env {
            "dev" | "development" => {}
            "prod" | "production" => {}
            "test" | "testing" => {}
            _ => {}
        }
        Ok(())
    }
}

