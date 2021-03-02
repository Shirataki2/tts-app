use actix_web::error;
use std::{io::Error as IoError, path::PathBuf};
use thiserror::Error;
use toml::de::Error as TomlDeserializationError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("File not Found: {0}\n{1:#?}")]
    FileNotFound(PathBuf, IoError),
    #[error("Failed to deserialize config file: {0}\n{1:#?}")]
    ConfigDeserializationError(PathBuf, TomlDeserializationError),
    #[error("Failed to deserialize json: \n{0:#?}")]
    JsonDeserializationError(#[from] serde_json::error::Error),
    #[error("Failed to spawn process\n{0:#?}")]
    CommandSpawnError(IoError),
    #[error("Non-zero exit code ({2:?})\nStdout: {0:#?}\nStderr: {1:#?}")]
    CommandError(String, String, Option<i32>),
    #[error("IO Error: {0:#?}")]
    IoError(#[from] IoError),
    #[error("Opus Error: {0:#?}")]
    OpusError(#[from] opus::Error),
    #[error("Token exchange error")]
    RequestTokenError(),
    #[error("Subprocess Error")]
    SubprocessError(),
    #[error("Database Error {0:?}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Invalid Header Error: {0:#?}")]
    InvalidHeaderError(#[from] http::header::InvalidHeaderValue),
    #[error("Crypt Error")]
    CryptError(#[from] pwhash::error::Error),
}

impl error::ResponseError for AppError {}
