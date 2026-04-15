use crate::application::error::AppError;
use crate::application::volume::dto::VolumeDto;
use crate::application::volume::mapper::VolumeMapper;
use crate::application::volume::traits::VolumeRepository;
use crate::application::PruneResultDto;
use std::sync::Arc;

pub struct VolumeService {
    repository: Arc<dyn VolumeRepository>,
}

impl VolumeService {
    pub fn new(repository: Arc<dyn VolumeRepository>) -> Self {
        VolumeService { repository }
    }

    pub async fn get_all_volumes(&self) -> Result<Vec<VolumeDto>, AppError> {
        let volumes = self.repository.get_all().await?;
        Ok(VolumeMapper::to_dto_list(&volumes))
    }

    pub async fn get_volume_by_name(&self, name: &str) -> Result<Option<VolumeDto>, AppError> {
        let volume = self.repository.get_by_name(name).await?;
        Ok(volume.map(|v| VolumeMapper::to_dto(&v)))
    }

    pub async fn delete_volume(&self, name: &str) -> Result<(), AppError> {
        self.repository.delete(name).await
    }

    pub async fn prune_volumes(&self) -> Result<PruneResultDto, AppError> {
        self.repository.prune().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::volume::traits::MockVolumeRepository;
    use crate::domain::{Volume, VolumeId};
    use std::sync::Arc;

    fn make_volume(id: &str, name: &str) -> Volume {
        Volume::new(
            VolumeId::new(id).unwrap(),
            name,
            "local",
            format!("/var/lib/docker/volumes/{}/_data", name),
        )
    }

    #[tokio::test]
    async fn test_get_all_volumes_returns_dtos() {
        let mut mock = MockVolumeRepository::new();
        mock.expect_get_all().returning(|| {
            Ok(vec![
                make_volume("vol1", "my-data"),
                make_volume("vol2", "pg-data"),
            ])
        });

        let service = VolumeService::new(Arc::new(mock));
        let result = service.get_all_volumes().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "my-data");
        assert_eq!(result[0].driver, "local");
        assert_eq!(result[1].name, "pg-data");
    }

    #[tokio::test]
    async fn test_get_volume_by_name_found() {
        let mut mock = MockVolumeRepository::new();
        mock.expect_get_by_name()
            .withf(|name| name == "my-data")
            .returning(|_| Ok(Some(make_volume("vol1", "my-data"))));

        let service = VolumeService::new(Arc::new(mock));
        let result = service.get_volume_by_name("my-data").await.unwrap();
        assert!(result.is_some());
        let dto = result.unwrap();
        assert_eq!(dto.name, "my-data");
        assert_eq!(dto.driver, "local");
        assert!(dto.can_delete);
    }

    #[tokio::test]
    async fn test_get_volume_by_name_not_found() {
        let mut mock = MockVolumeRepository::new();
        mock.expect_get_by_name().returning(|_| Ok(None));

        let service = VolumeService::new(Arc::new(mock));
        let result = service.get_volume_by_name("missing").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_volume_delegates() {
        let mut mock = MockVolumeRepository::new();
        mock.expect_delete()
            .withf(|name| name == "my-data")
            .returning(|_| Ok(()));

        let service = VolumeService::new(Arc::new(mock));
        assert!(service.delete_volume("my-data").await.is_ok());
    }

    #[tokio::test]
    async fn test_prune_volumes_returns_result() {
        let mut mock = MockVolumeRepository::new();
        mock.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 2,
                space_freed: 500_000,
            })
        });

        let service = VolumeService::new(Arc::new(mock));
        let result = service.prune_volumes().await.unwrap();
        assert_eq!(result.deleted_count, 2);
        assert_eq!(result.space_freed, 500_000);
    }
}
