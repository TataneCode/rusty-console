#[derive(Debug, Clone)]
pub struct PruneResultDto {
    pub deleted_count: u32,
    pub space_freed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteSize(i64);

impl ByteSize {
    pub fn new(bytes: i64) -> Self {
        ByteSize(bytes)
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

impl Default for ByteSize {
    fn default() -> Self {
        ByteSize(-1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_size_human_readable() {
        assert_eq!(ByteSize::new(500).human_readable(), "500 B");
        assert_eq!(ByteSize::new(1536).human_readable(), "1.50 KB");
        assert_eq!(ByteSize::new(1_572_864).human_readable(), "1.50 MB");
        assert_eq!(ByteSize::new(1_610_612_736).human_readable(), "1.50 GB");
        assert_eq!(ByteSize::new(-1).human_readable(), "N/A");
    }

    #[test]
    fn test_byte_size_default_is_na() {
        assert_eq!(ByteSize::default().human_readable(), "N/A");
    }

    #[test]
    fn test_byte_size_bytes() {
        assert_eq!(ByteSize::new(1024).bytes(), 1024);
    }
}
