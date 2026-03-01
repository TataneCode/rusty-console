use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invalid container ID: {0}")]
    InvalidContainerId(String),

    #[error("Invalid volume ID: {0}")]
    InvalidVolumeId(String),

    #[error("Invalid image ID: {0}")]
    InvalidImageId(String),

    #[error("Invalid state transition: cannot {action} container in {current_state} state")]
    InvalidStateTransition {
        action: String,
        current_state: String,
    },
}
