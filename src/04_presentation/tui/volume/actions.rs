use crate::application::error::AppError;
use crate::application::volume::{VolumeDto, VolumeService};
use crate::shared::PruneResultDto;

pub struct VolumeActions {
    service: VolumeService,
}

impl VolumeActions {
    pub fn new(service: VolumeService) -> Self {
        VolumeActions { service }
    }

    pub async fn load_volumes(&self) -> Result<Vec<VolumeDto>, AppError> {
        self.service.get_all_volumes().await
    }

    pub async fn delete_volume(&self, name: &str) -> Result<(), AppError> {
        self.service.delete_volume(name).await
    }

    pub async fn prune_volumes(&self) -> Result<PruneResultDto, AppError> {
        self.service.prune_volumes().await
    }
}

#[cfg(test)]
mod tests {
    use super::VolumeActions;
    use crate::application::volume::traits::MockVolumeRepository;
    use crate::application::volume::VolumeService;
    use crate::domain::volume::{Volume, VolumeId};
    use crate::shared::PruneResultDto;
    use std::sync::Arc;

    fn make_volume() -> Volume {
        Volume::new(
            VolumeId::new("vol-1").unwrap(),
            "db-data",
            "local",
            "/var/lib/docker/volumes/db-data/_data".to_string(),
        )
    }

    #[tokio::test]
    async fn test_volume_actions_delegate_all_operations() {
        let mut mock = MockVolumeRepository::new();
        mock.expect_get_all().returning(|| Ok(vec![make_volume()]));
        mock.expect_delete().returning(|_| Ok(()));
        mock.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 1,
                space_freed: 512,
            })
        });

        let actions = VolumeActions::new(VolumeService::new(Arc::new(mock)));
        let volumes = actions.load_volumes().await.unwrap();

        assert_eq!(volumes.len(), 1);
        assert!(actions.delete_volume(&volumes[0].name).await.is_ok());
        assert_eq!(actions.prune_volumes().await.unwrap().space_freed, 512);
    }
}
