use crate::domain::{Volume, VolumeId, VolumeSize};
use bollard::models::Volume as BollardVolume;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct VolumeInfraMapper;

impl VolumeInfraMapper {
    pub fn from_docker(
        volume: &BollardVolume,
        size_map: &HashMap<String, i64>,
        in_use_volumes: &[String],
    ) -> Option<Volume> {
        let name = &volume.name;
        let id = VolumeId::new(name.clone()).ok()?;

        let driver = volume.driver.clone();
        let mountpoint = volume.mountpoint.clone();

        let size = size_map
            .get(name)
            .copied()
            .map(VolumeSize::new)
            .unwrap_or_default();

        let created = volume
            .created_at
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let in_use = in_use_volumes.contains(name);

        let mut vol = Volume::new(id, name.clone(), driver, mountpoint)
            .with_size(size)
            .with_in_use(in_use);

        if let Some(created) = created {
            vol = vol.with_created(created);
        }

        Some(vol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::Volume as BollardVolume;

    fn make_bollard_volume(name: &str) -> BollardVolume {
        BollardVolume {
            name: name.to_string(),
            driver: "local".to_string(),
            mountpoint: format!("/var/lib/docker/volumes/{name}/_data"),
            created_at: Some("2024-01-15T10:30:00Z".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn from_docker_normal_volume_with_size() {
        let vol = make_bollard_volume("pgdata");
        let mut size_map = HashMap::new();
        size_map.insert("pgdata".to_string(), 1_048_576);
        let in_use = vec!["pgdata".to_string()];

        let volume = VolumeInfraMapper::from_docker(&vol, &size_map, &in_use).unwrap();
        assert_eq!(volume.name(), "pgdata");
        assert_eq!(volume.driver(), "local");
        assert_eq!(volume.size().bytes(), 1_048_576);
        assert!(volume.in_use());
        assert!(volume.created().is_some());
        assert!(!volume.can_be_deleted());
    }

    #[test]
    fn from_docker_volume_not_in_size_map_not_in_use() {
        let vol = make_bollard_volume("temp_vol");
        let size_map = HashMap::new();
        let in_use: Vec<String> = vec![];

        let volume = VolumeInfraMapper::from_docker(&vol, &size_map, &in_use).unwrap();
        assert_eq!(volume.name(), "temp_vol");
        assert_eq!(volume.size().bytes(), -1); // VolumeSize default is -1 (unknown)
        assert!(!volume.in_use());
        assert!(volume.can_be_deleted());
    }
}
