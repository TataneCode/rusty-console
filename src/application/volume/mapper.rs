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
