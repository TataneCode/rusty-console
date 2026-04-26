use crate::application::error::AppError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfraError {
    #[error("Docker API error: {0}")]
    Docker(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<bollard::errors::Error> for InfraError {
    fn from(err: bollard::errors::Error) -> Self {
        InfraError::Docker(err.to_string())
    }
}

impl From<InfraError> for AppError {
    fn from(err: InfraError) -> Self {
        match err {
            InfraError::Docker(msg) => AppError::repository(msg),
            InfraError::Connection(msg) => AppError::connection(msg),
            InfraError::Serialization(msg) => AppError::repository(msg),
        }
    }
}
