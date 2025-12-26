use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Cargo error: {0}")]
    Cargo(#[from] std::io::Error),
    #[error("Metadata error: {0}")]
    Metadata(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
