use crate::errors::DomainError;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeSize(i64);

impl VolumeSize {
    pub fn new(bytes: i64) -> Self {
        VolumeSize(bytes)
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

impl Default for VolumeSize {
    fn default() -> Self {
        VolumeSize(-1)
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

    #[test]
    fn test_volume_size_human_readable() {
        assert_eq!(VolumeSize::new(500).human_readable(), "500 B");
        assert_eq!(VolumeSize::new(1536).human_readable(), "1.50 KB");
        assert_eq!(VolumeSize::new(1_572_864).human_readable(), "1.50 MB");
        assert_eq!(VolumeSize::new(1_610_612_736).human_readable(), "1.50 GB");
        assert_eq!(VolumeSize::new(-1).human_readable(), "N/A");
    }
}
