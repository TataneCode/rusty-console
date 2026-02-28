use crate::application::error::AppError;
use crate::application::volume::dto::VolumeDto;
use crate::application::volume::mapper::VolumeMapper;
use crate::application::volume::traits::VolumeRepository;
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
}
