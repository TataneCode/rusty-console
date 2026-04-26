use crate::application::error::AppError;
use crate::application::image::{ImageDto, ImageService};
use crate::shared::PruneResultDto;

pub struct ImageActions {
    service: ImageService,
}

impl ImageActions {
    pub fn new(service: ImageService) -> Self {
        ImageActions { service }
    }

    pub async fn load_images(&self) -> Result<Vec<ImageDto>, AppError> {
        self.service.get_all_images().await
    }

    pub async fn delete_image(&self, id: &str, force: bool) -> Result<(), AppError> {
        self.service.delete_image(id, force).await
    }

    pub async fn prune_images(&self) -> Result<PruneResultDto, AppError> {
        self.service.prune_images().await
    }
}
