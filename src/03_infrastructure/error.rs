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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_error_maps_to_repository() {
        let err = InfraError::Docker("timeout".to_string());
        let app_err = AppError::from(err);
        assert!(matches!(app_err, AppError::Repository(_)));
        assert!(app_err.to_string().contains("timeout"));
    }

    #[test]
    fn test_connection_error_maps_to_connection() {
        let err = InfraError::Connection("refused".to_string());
        let app_err = AppError::from(err);
        assert!(matches!(app_err, AppError::Connection(_)));
        assert!(app_err.to_string().contains("refused"));
    }

    #[test]
    fn test_serialization_error_maps_to_repository() {
        let err = InfraError::Serialization("bad json".to_string());
        let app_err = AppError::from(err);
        assert!(matches!(app_err, AppError::Repository(_)));
        assert!(app_err.to_string().contains("bad json"));
    }
}
