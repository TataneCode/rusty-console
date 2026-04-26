use crate::domain::error::DomainError;

pub type ImageSize = crate::shared::ByteSize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageId(String);

impl ImageId {
    pub fn new(id: impl Into<String>) -> Result<Self, DomainError> {
        let id = id.into();
        if id.is_empty() {
            return Err(DomainError::InvalidImageId(
                "Image ID cannot be empty".to_string(),
            ));
        }
        Ok(ImageId(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn short(&self) -> &str {
        let id = self.0.strip_prefix("sha256:").unwrap_or(&self.0);
        if id.len() > 12 {
            &id[..12]
        } else {
            id
        }
    }
}

impl std::fmt::Display for ImageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_id_valid() {
        let id = ImageId::new("sha256:abc123def456").unwrap();
        assert_eq!(id.as_str(), "sha256:abc123def456");
    }

    #[test]
    fn test_image_id_short() {
        let id = ImageId::new("sha256:abcdef1234567890abcdef").unwrap();
        assert_eq!(id.short(), "abcdef123456");

        let id_no_prefix = ImageId::new("abcdef1234567890").unwrap();
        assert_eq!(id_no_prefix.short(), "abcdef123456");
    }

    #[test]
    fn test_image_id_empty() {
        let result = ImageId::new("");
        assert!(result.is_err());
    }
}
