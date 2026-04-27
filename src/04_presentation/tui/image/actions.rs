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

#[cfg(test)]
mod tests {
    use super::ImageActions;
    use crate::application::image::traits::MockImageRepository;
    use crate::application::image::ImageService;
    use crate::domain::image::{Image, ImageId, ImageSize};
    use crate::shared::PruneResultDto;
    use chrono::Utc;
    use std::sync::Arc;

    fn make_image() -> Image {
        Image::new(
            ImageId::new("sha256:abc").unwrap(),
            "nginx",
            "latest",
            ImageSize::new(10_000),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn test_image_actions_delegate_all_operations() {
        let mut mock = MockImageRepository::new();
        mock.expect_get_all().returning(|| Ok(vec![make_image()]));
        mock.expect_delete().returning(|_, _| Ok(()));
        mock.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 4,
                space_freed: 2048,
            })
        });

        let actions = ImageActions::new(ImageService::new(Arc::new(mock)));
        let images = actions.load_images().await.unwrap();

        assert_eq!(images.len(), 1);
        assert!(actions.delete_image(&images[0].id, true).await.is_ok());
        assert_eq!(actions.prune_images().await.unwrap().deleted_count, 4);
    }
}
