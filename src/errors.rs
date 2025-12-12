use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum ConsulXError {
    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Missing argument: {0}")]
    MissingArgument(&'static str),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ConsulXError>;

// Allow `?` on serde_json errors
impl From<serde_json::Error> for ConsulXError {
    fn from(e: serde_json::Error) -> Self {
        ConsulXError::Other(anyhow::Error::new(e))
    }
}

// Allow `?` on std::io errors (fs::write, read_to_string, Command::status, etc.)
impl From<io::Error> for ConsulXError {
    fn from(e: io::Error) -> Self {
        ConsulXError::Other(anyhow::Error::new(e))
    }
}
