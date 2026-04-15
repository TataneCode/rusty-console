use crate::application::error::AppError;
use crate::application::image::dto::ImageDto;
use crate::application::image::mapper::ImageMapper;
use crate::application::image::traits::ImageRepository;
use crate::application::PruneResultDto;
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

    pub async fn prune_images(&self) -> Result<PruneResultDto, AppError> {
        self.repository.prune().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::image::traits::MockImageRepository;
    use crate::domain::{Image, ImageId, ImageSize};
    use chrono::Utc;
    use std::sync::Arc;

    fn make_image(id: &str, repo: &str, tag: &str) -> Image {
        Image::new(
            ImageId::new(id).unwrap(),
            repo,
            tag,
            ImageSize::new(1_000_000),
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn test_get_all_images_returns_dtos() {
        let mut mock = MockImageRepository::new();
        mock.expect_get_all().returning(|| {
            Ok(vec![
                make_image("sha256:aaa", "nginx", "latest"),
                make_image("sha256:bbb", "redis", "7"),
            ])
        });

        let service = ImageService::new(Arc::new(mock));
        let result = service.get_all_images().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].repository, "nginx");
        assert_eq!(result[0].tag, "latest");
        assert_eq!(result[0].full_name, "nginx:latest");
        assert_eq!(result[1].repository, "redis");
    }

    #[tokio::test]
    async fn test_get_image_by_id_found() {
        let mut mock = MockImageRepository::new();
        mock.expect_get_by_id()
            .withf(|id| id == "sha256:aaa")
            .returning(|_| Ok(Some(make_image("sha256:aaa", "nginx", "latest"))));

        let service = ImageService::new(Arc::new(mock));
        let result = service.get_image_by_id("sha256:aaa").await.unwrap();
        assert!(result.is_some());
        let dto = result.unwrap();
        assert_eq!(dto.repository, "nginx");
        assert_eq!(dto.tag, "latest");
        assert!(!dto.in_use);
        assert!(dto.can_delete);
    }

    #[tokio::test]
    async fn test_get_image_by_id_not_found() {
        let mut mock = MockImageRepository::new();
        mock.expect_get_by_id().returning(|_| Ok(None));

        let service = ImageService::new(Arc::new(mock));
        let result = service.get_image_by_id("missing").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_image_delegates() {
        let mut mock = MockImageRepository::new();
        mock.expect_delete()
            .withf(|id, force| id == "sha256:aaa" && *force)
            .returning(|_, _| Ok(()));

        let service = ImageService::new(Arc::new(mock));
        assert!(service.delete_image("sha256:aaa", true).await.is_ok());
    }

    #[tokio::test]
    async fn test_prune_images_returns_result() {
        let mut mock = MockImageRepository::new();
        mock.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 5,
                space_freed: 2_000_000,
            })
        });

        let service = ImageService::new(Arc::new(mock));
        let result = service.prune_images().await.unwrap();
        assert_eq!(result.deleted_count, 5);
        assert_eq!(result.space_freed, 2_000_000);
    }
}
