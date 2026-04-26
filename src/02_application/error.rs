use crate::domain::error::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

impl AppError {
    pub fn repository(msg: impl Into<String>) -> Self {
        AppError::Repository(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        AppError::NotFound(msg.into())
    }

    pub fn operation_failed(msg: impl Into<String>) -> Self {
        AppError::OperationFailed(msg.into())
    }

    pub fn connection(msg: impl Into<String>) -> Self {
        AppError::Connection(msg.into())
    }
}
