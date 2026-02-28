use crate::application::error::AppError;
use crate::application::image::dto::ImageDto;
use crate::application::image::mapper::ImageMapper;
use crate::application::image::traits::ImageRepository;
use std::sync::Arc;

pub struct ImageService {
    repository: Arc<dyn ImageRepository>,
}

impl ImageService {
    pub fn new(repository: Arc<dyn ImageRepository>) -> Self {
        ImageService { repository }
    }

    pub async fn get_all_images(&self) -> Result<Vec<ImageDto>, AppError> {
        let images = self.repository.get_all().await?;
        Ok(ImageMapper::to_dto_list(&images))
    }

    pub async fn get_image_by_id(&self, id: &str) -> Result<Option<ImageDto>, AppError> {
        let image = self.repository.get_by_id(id).await?;
        Ok(image.map(|i| ImageMapper::to_dto(&i)))
    }

    pub async fn delete_image(&self, id: &str, force: bool) -> Result<(), AppError> {
        self.repository.delete(id, force).await
    }
}
