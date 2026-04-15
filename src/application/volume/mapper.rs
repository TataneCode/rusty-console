use crate::application::volume::dto::VolumeDto;
use crate::domain::Volume;

pub struct VolumeMapper;

impl VolumeMapper {
    pub fn to_dto(volume: &Volume) -> VolumeDto {
        VolumeDto {
            id: volume.id().to_string(),
            name: volume.name().to_string(),
            driver: volume.driver().to_string(),
            mountpoint: volume.mountpoint().to_string(),
            size: volume.size_display(),
            created: volume
                .created()
                .map(|c| c.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            in_use: volume.in_use(),
            can_delete: volume.can_be_deleted(),
        }
    }

    pub fn to_dto_list(volumes: &[Volume]) -> Vec<VolumeDto> {
        volumes.iter().map(Self::to_dto).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{VolumeId, VolumeSize};
    use chrono::{TimeZone, Utc};

    fn create_test_volume() -> Volume {
        Volume::new(
            VolumeId::new("vol-abc123").unwrap(),
            "my-data",
            "local",
            "/var/lib/docker/volumes/my-data/_data",
        )
        .with_size(VolumeSize::new(5_242_880))
        .with_created(Utc.with_ymd_and_hms(2024, 6, 1, 9, 0, 0).unwrap())
    }

    #[test]
    fn to_dto_maps_basic_fields() {
        let volume = create_test_volume();
        let dto = VolumeMapper::to_dto(&volume);

        assert_eq!(dto.id, "vol-abc123");
        assert_eq!(dto.name, "my-data");
        assert_eq!(dto.driver, "local");
        assert_eq!(dto.mountpoint, "/var/lib/docker/volumes/my-data/_data");
        assert_eq!(dto.size, "5.00 MB");
        assert_eq!(dto.created, "2024-06-01 09:00:00");
    }

    #[test]
    fn to_dto_not_in_use_can_delete() {
        let volume = create_test_volume().with_in_use(false);
        let dto = VolumeMapper::to_dto(&volume);

        assert!(!dto.in_use);
        assert!(dto.can_delete);
    }

    #[test]
    fn to_dto_in_use_cannot_delete() {
        let volume = create_test_volume().with_in_use(true);
        let dto = VolumeMapper::to_dto(&volume);

        assert!(dto.in_use);
        assert!(!dto.can_delete);
    }

    #[test]
    fn to_dto_no_created_date_shows_na() {
        let volume = Volume::new(
            VolumeId::new("vol-no-date").unwrap(),
            "ephemeral",
            "local",
            "/mnt",
        );
        let dto = VolumeMapper::to_dto(&volume);
        assert_eq!(dto.created, "N/A");
    }

    #[test]
    fn to_dto_default_size_shows_na() {
        let volume = Volume::new(
            VolumeId::new("vol-nosize").unwrap(),
            "empty-vol",
            "local",
            "/mnt",
        );
        let dto = VolumeMapper::to_dto(&volume);
        assert_eq!(dto.size, "N/A");
    }

    #[test]
    fn to_dto_list_maps_multiple_volumes() {
        let volumes = vec![
            create_test_volume(),
            Volume::new(
                VolumeId::new("vol-second").unwrap(),
                "db-data",
                "nfs",
                "/mnt/nfs/db",
            )
            .with_in_use(true),
        ];
        let dtos = VolumeMapper::to_dto_list(&volumes);

        assert_eq!(dtos.len(), 2);
        assert_eq!(dtos[0].name, "my-data");
        assert_eq!(dtos[1].name, "db-data");
        assert_eq!(dtos[1].driver, "nfs");
        assert!(dtos[1].in_use);
    }

    #[test]
    fn to_dto_list_empty_input() {
        let dtos = VolumeMapper::to_dto_list(&[]);
        assert!(dtos.is_empty());
    }
}
