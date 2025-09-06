use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Operation failed: {0}")]
    Operation(String),
}

pub type Result<T> = std::result::Result<T, Error>;

