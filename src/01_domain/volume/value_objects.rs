use crate::domain::error::DomainError;

pub type VolumeSize = crate::shared::ByteSize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VolumeId(String);

impl VolumeId {
    pub fn new(id: impl Into<String>) -> Result<Self, DomainError> {
        let id = id.into();
        if id.is_empty() {
            return Err(DomainError::InvalidVolumeId(
                "Volume ID cannot be empty".to_string(),
            ));
        }
        Ok(VolumeId(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for VolumeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_id_valid() {
        let id = VolumeId::new("my-volume").unwrap();
        assert_eq!(id.as_str(), "my-volume");
    }

    #[test]
    fn test_volume_id_empty() {
        let result = VolumeId::new("");
        assert!(result.is_err());
    }
}
