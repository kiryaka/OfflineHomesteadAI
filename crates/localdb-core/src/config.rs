use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use std::env;
use std::path::{Path, PathBuf};

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

/// Expand a user-provided path string:
/// - Expands leading '~' to the user's home directory
/// - Expands ${VAR} and $VAR environment variables
/// - Returns a PathBuf without attempting to canonicalize
pub fn expand_path<S: AsRef<str>>(input: S) -> PathBuf {
    let s = input.as_ref();
    // Expand env vars first
    let expanded_env = shellexpand::env(s).unwrap_or(std::borrow::Cow::Borrowed(s));
    // Expand ~ at start
    let expanded = shellexpand::tilde(&expanded_env);
    PathBuf::from(expanded.as_ref())
}

/// Resolve a possibly relative path against a given base directory after expansion.
/// If `p` is absolute, it's returned as-is; otherwise `base.join(p)` is returned.
pub fn resolve_with_base<S: AsRef<str>>(base: &Path, p: S) -> PathBuf {
    let p = expand_path(p);
    if p.is_absolute() { p } else { base.join(p) }
}
//! Lightweight configuration loader and path helpers.
//!
//! Uses Figment to merge `config.toml` + `config.<env>.toml` + `APP_*` env vars.
//! Provides helpers to expand `~` and `${VAR}` and to resolve relative paths
//! against a known base directory.
