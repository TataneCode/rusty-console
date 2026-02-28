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
