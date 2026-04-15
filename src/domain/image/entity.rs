use super::value_objects::{ImageId, ImageSize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Image {
    id: ImageId,
    repository: String,
    tag: String,
    size: ImageSize,
    created: DateTime<Utc>,
    in_use: bool,
}

impl Image {
    pub fn new(
        id: ImageId,
        repository: impl Into<String>,
        tag: impl Into<String>,
        size: ImageSize,
        created: DateTime<Utc>,
    ) -> Self {
        Image {
            id,
            repository: repository.into(),
            tag: tag.into(),
            size,
            created,
            in_use: false,
        }
    }

    pub fn with_in_use(mut self, in_use: bool) -> Self {
        self.in_use = in_use;
        self
    }

    // Getters
    pub fn id(&self) -> &ImageId {
        &self.id
    }

    pub fn repository(&self) -> &str {
        &self.repository
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn size(&self) -> ImageSize {
        self.size
    }

    pub fn created(&self) -> DateTime<Utc> {
        self.created
    }

    pub fn in_use(&self) -> bool {
        self.in_use
    }

    // Business logic
    pub fn can_be_deleted(&self) -> bool {
        !self.in_use
    }

    pub fn full_name(&self) -> String {
        if self.tag.is_empty() || self.tag == "<none>" {
            self.repository.clone()
        } else if self.repository.is_empty() || self.repository == "<none>" {
            format!("<none>:{}", self.tag)
        } else {
            format!("{}:{}", self.repository, self.tag)
        }
    }

    pub fn size_display(&self) -> String {
        self.size.human_readable()
    }

    pub fn is_dangling(&self) -> bool {
        (self.repository.is_empty() || self.repository == "<none>")
            && (self.tag.is_empty() || self.tag == "<none>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(repo: &str, tag: &str) -> Image {
        Image::new(
            ImageId::new("sha256:test123").unwrap(),
            repo,
            tag,
            ImageSize::new(1_000_000),
            Utc::now(),
        )
    }

    #[test]
    fn test_full_name() {
        let image = create_test_image("nginx", "latest");
        assert_eq!(image.full_name(), "nginx:latest");
    }

    #[test]
    fn test_full_name_no_tag() {
        let image = create_test_image("nginx", "<none>");
        assert_eq!(image.full_name(), "nginx");
    }

    #[test]
    fn test_is_dangling() {
        let dangling = create_test_image("<none>", "<none>");
        assert!(dangling.is_dangling());

        let normal = create_test_image("nginx", "latest");
        assert!(!normal.is_dangling());
    }

    #[test]
    fn test_can_be_deleted() {
        let image = create_test_image("nginx", "latest");
        assert!(image.can_be_deleted());

        let in_use = create_test_image("nginx", "latest").with_in_use(true);
        assert!(!in_use.can_be_deleted());
    }

    #[test]
    fn test_can_be_deleted_explicitly_not_in_use() {
        let image = create_test_image("nginx", "latest").with_in_use(false);
        assert!(image.can_be_deleted());
    }

    #[test]
    fn test_short_id() {
        let image = create_test_image("nginx", "latest");
        assert_eq!(image.id().short(), "test123");
    }

    #[test]
    fn test_full_name_no_repo() {
        let image = create_test_image("<none>", "latest");
        assert_eq!(image.full_name(), "<none>:latest");
    }

    #[test]
    fn test_is_dangling_empty_strings() {
        let image = create_test_image("", "");
        assert!(image.is_dangling());
    }
}
