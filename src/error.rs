#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    Usage(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("command failed: {0}")]
    Command(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) | Self::Config(_) => 2,
            Self::Command(_) | Self::Io(_) => 1,
        }
    }
}
