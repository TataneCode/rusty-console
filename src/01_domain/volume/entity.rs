use super::value_objects::{VolumeId, VolumeSize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Volume {
    id: VolumeId,
    name: String,
    driver: String,
    mountpoint: String,
    size: VolumeSize,
    created: Option<DateTime<Utc>>,
    in_use: bool,
    linked_containers: Vec<String>,
}

impl Volume {
    pub fn new(
        id: VolumeId,
        name: impl Into<String>,
        driver: impl Into<String>,
        mountpoint: impl Into<String>,
    ) -> Self {
        Volume {
            id,
            name: name.into(),
            driver: driver.into(),
            mountpoint: mountpoint.into(),
            size: VolumeSize::default(),
            created: None,
            in_use: false,
            linked_containers: Vec::new(),
        }
    }

    pub fn with_size(mut self, size: VolumeSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_created(mut self, created: DateTime<Utc>) -> Self {
        self.created = Some(created);
        self
    }

    pub fn with_in_use(mut self, in_use: bool) -> Self {
        self.in_use = in_use;
        self
    }

    pub fn with_linked_containers(mut self, linked_containers: Vec<String>) -> Self {
        self.in_use = !linked_containers.is_empty();
        self.linked_containers = linked_containers;
        self
    }

    // Getters
    pub fn id(&self) -> &VolumeId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn driver(&self) -> &str {
        &self.driver
    }

    pub fn mountpoint(&self) -> &str {
        &self.mountpoint
    }

    pub fn size(&self) -> VolumeSize {
        self.size
    }

    pub fn created(&self) -> Option<DateTime<Utc>> {
        self.created
    }

    pub fn in_use(&self) -> bool {
        self.in_use
    }

    pub fn linked_containers(&self) -> &[String] {
        &self.linked_containers
    }

    // Business logic
    pub fn can_be_deleted(&self) -> bool {
        !self.in_use
    }

    pub fn size_display(&self) -> String {
        self.size.human_readable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_be_deleted_when_not_in_use() {
        let volume = Volume::new(
            VolumeId::new("vol1").unwrap(),
            "my-volume",
            "local",
            "/var/lib/docker/volumes/my-volume/_data",
        )
        .with_in_use(false);

        assert!(volume.can_be_deleted());
    }

    #[test]
    fn test_cannot_be_deleted_when_in_use() {
        let volume = Volume::new(
            VolumeId::new("vol1").unwrap(),
            "my-volume",
            "local",
            "/var/lib/docker/volumes/my-volume/_data",
        )
        .with_in_use(true);

        assert!(!volume.can_be_deleted());
    }

    #[test]
    fn test_linked_containers_mark_volume_as_in_use() {
        let volume = Volume::new(
            VolumeId::new("vol1").unwrap(),
            "my-volume",
            "local",
            "/var/lib/docker/volumes/my-volume/_data",
        )
        .with_linked_containers(vec!["web".to_string(), "worker".to_string()]);

        assert!(volume.in_use());
        assert_eq!(
            volume.linked_containers(),
            &["web".to_string(), "worker".to_string()]
        );
        assert!(!volume.can_be_deleted());
    }

    fn create_test_volume() -> Volume {
        Volume::new(
            VolumeId::new("vol1").unwrap(),
            "my-volume",
            "local",
            "/var/lib/docker/volumes/my-volume/_data",
        )
    }

    #[test]
    fn test_name() {
        let volume = create_test_volume();
        assert_eq!(volume.name(), "my-volume");
    }

    #[test]
    fn test_mountpoint() {
        let volume = create_test_volume();
        assert_eq!(
            volume.mountpoint(),
            "/var/lib/docker/volumes/my-volume/_data"
        );
    }

    #[test]
    fn test_driver() {
        let volume = create_test_volume();
        assert_eq!(volume.driver(), "local");
    }
}
