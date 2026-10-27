use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invalid container ID: {0}")]
    InvalidContainer(String),

    #[error("Invalid volume ID: {0}")]
    InvalidVolume(String),

    #[error("Invalid network ID: {0}")]
    InvalidNetwork(String),

    #[error("Invalid image ID: {0}")]
    InvalidImage(String),

    #[error("Invalid stack name: {0}")]
    InvalidStack(String),

    #[error("Invalid state transition: cannot {action} container in {current_state} state")]
    InvalidStateTransition {
        action: String,
        current_state: String,
    },

    #[error("Invalid file entry: {0}")]
    InvalidFileEntry(String),

    #[error("Invalid file listing: {0}")]
    InvalidFileListing(String),
}
