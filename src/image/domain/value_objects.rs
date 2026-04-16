use crate::errors::DomainError;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageSize(i64);

impl ImageSize {
    pub fn new(bytes: i64) -> Self {
        ImageSize(bytes)
    }

    pub fn bytes(&self) -> i64 {
        self.0
    }

    pub fn human_readable(&self) -> String {
        const KB: i64 = 1024;
        const MB: i64 = KB * 1024;
        const GB: i64 = MB * 1024;

        if self.0 < 0 {
            return "N/A".to_string();
        }

        if self.0 >= GB {
            format!("{:.2} GB", self.0 as f64 / GB as f64)
        } else if self.0 >= MB {
            format!("{:.2} MB", self.0 as f64 / MB as f64)
        } else if self.0 >= KB {
            format!("{:.2} KB", self.0 as f64 / KB as f64)
        } else {
            format!("{} B", self.0)
        }
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

    #[test]
    fn test_image_size_human_readable() {
        assert_eq!(ImageSize::new(500).human_readable(), "500 B");
        assert_eq!(ImageSize::new(1_048_576).human_readable(), "1.00 MB");
        assert_eq!(ImageSize::new(1_073_741_824).human_readable(), "1.00 GB");
    }
}
