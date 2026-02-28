use crate::application::{AppError, VolumeDto, VolumeService};

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
}
