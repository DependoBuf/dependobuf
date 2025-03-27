use super::page::PageId;

use bincode::error::{DecodeError, EncodeError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Serialization error: {0}")]
    EncodeError(#[from] EncodeError),

    #[error("Deserialization error: {0}")]
    DecodeError(#[from] DecodeError),

    #[error("Page not found: {0}")]
    PageNotFound(PageId),

    #[error("Page full")]
    PageFull,

    #[error("Invalid operation")]
    InvalidOperation,

    #[error("IO error")]
    IOError(#[from] std::io::Error),
}
