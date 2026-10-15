use crate::application::error::AppError;
use crate::application::image::dto::ImagePullEvent;
use crate::application::image::dto::{ImageDto, ImagePullOutcome, ImagePullSubscription};
use crate::application::image::mapper::ImageMapper;
use crate::application::image::traits::ImageRepository;
use crate::application::shared::PruneResultDto;
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

    pub async fn pull_images(
        &self,
        image_refs: &[String],
    ) -> Result<ImagePullSubscription, AppError> {
        let mut to_pull: Vec<String> = Vec::new();
        let mut up_to_date: Vec<String> = Vec::new();
        for image_ref in image_refs {
            if self.is_up_to_date(image_ref).await? {
                up_to_date.push(image_ref.clone());
            } else {
                to_pull.push(image_ref.clone());
            }
        }
        if to_pull.is_empty() {
            return Ok(ImagePullSubscription::completed(Ok(ImagePullOutcome {
                pulled: Vec::new(),
                up_to_date,
            })));
        }
        let subscription = self.repository.pull_images(&to_pull).await?;
        Ok(with_outcome_up_to_date(subscription, up_to_date))
    }

    async fn is_up_to_date(&self, image_ref: &str) -> Result<bool, AppError> {
        let Some(local) = self.repository.local_digest(image_ref).await? else {
            return Ok(false);
        };
        let Some(remote) = self.repository.registry_digest(image_ref).await? else {
            return Ok(false);
        };
        Ok(local == remote)
    }
}

fn with_outcome_up_to_date(
    mut subscription: ImagePullSubscription,
    up_to_date: Vec<String>,
) -> ImagePullSubscription {
    if up_to_date.is_empty() {
        return subscription;
    }
    let (sender, receiver) = tokio::sync::mpsc::channel(128);
    let relay = tokio::spawn(async move {
        loop {
            match subscription.try_recv() {
                Ok(ImagePullEvent::Progress(progress)) => {
                    if sender
                        .send(ImagePullEvent::Progress(progress))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                Ok(ImagePullEvent::Completed(Ok(mut outcome))) => {
                    let mut merged = up_to_date;
                    merged.append(&mut outcome.up_to_date);
                    let _ = sender
                        .send(ImagePullEvent::Completed(Ok(ImagePullOutcome {
                            pulled: outcome.pulled,
                            up_to_date: merged,
                        })))
                        .await;
                    return;
                }
                Ok(ImagePullEvent::Completed(Err(error))) => {
                    let _ = sender.send(ImagePullEvent::Completed(Err(error))).await;
                    return;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    let _ = sender
                        .send(ImagePullEvent::Completed(Err(
                            crate::application::error::AppError::operation_failed(
                                "Image pull stream ended unexpectedly".to_string(),
                            ),
                        )))
                        .await;
                    return;
                }
            }
        }
    });
    ImagePullSubscription::new(receiver, vec![relay])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::image::dto::{ImagePullEvent, ImagePullOutcome, ImagePullSubscription};
    use crate::application::image::traits::MockImageRepository;
    use crate::domain::image::{Image, ImageId, ImageSize};
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
    async fn test_pull_images_skips_up_to_date_images() {
        let mut mock = MockImageRepository::new();
        mock.expect_local_digest().times(2).returning(|image_ref| {
            if image_ref == "nginx:latest" {
                Ok(Some("sha256:aaa".to_string()))
            } else {
                Ok(None)
            }
        });
        mock.expect_registry_digest().returning(|image_ref| {
            if image_ref == "nginx:latest" {
                Ok(Some("sha256:aaa".to_string()))
            } else {
                Ok(None)
            }
        });
        mock.expect_pull_images().returning(|_| {
            Ok(ImagePullSubscription::completed(Ok(ImagePullOutcome {
                pulled: vec!["redis:7".to_string()],
                up_to_date: vec![],
            })))
        });
        let service = ImageService::new(Arc::new(mock));
        let mut subscription = service
            .pull_images(&["nginx:latest".to_string(), "redis:7".to_string()])
            .await
            .unwrap();
        let outcome = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                match subscription.try_recv() {
                    Ok(ImagePullEvent::Completed(result)) => break result.unwrap(),
                    Ok(ImagePullEvent::Progress(_)) => {}
                    Err(_) => tokio::time::sleep(std::time::Duration::from_millis(10)).await,
                }
            }
        })
        .await
        .expect("pull completion timed out");
        assert_eq!(outcome.pulled, vec!["redis:7".to_string()]);
        assert_eq!(outcome.up_to_date, vec!["nginx:latest".to_string()]);
    }

    #[tokio::test]
    async fn test_pull_images_all_up_to_date_completes_without_repository_pull() {
        let mut mock = MockImageRepository::new();
        mock.expect_local_digest()
            .returning(|_| Ok(Some("sha256:bbb".to_string())));
        mock.expect_registry_digest()
            .returning(|_| Ok(Some("sha256:bbb".to_string())));
        let service = ImageService::new(Arc::new(mock));
        let mut subscription = service
            .pull_images(&["nginx:latest".to_string()])
            .await
            .unwrap();
        match subscription.try_recv().unwrap() {
            ImagePullEvent::Completed(result) => {
                let outcome = result.unwrap();
                assert!(outcome.pulled.is_empty());
                assert_eq!(outcome.up_to_date, vec!["nginx:latest".to_string()]);
            }
            _ => panic!("expected completed event"),
        }
    }

    #[tokio::test]
    async fn test_pull_images_pulls_when_digests_differ() {
        let mut mock = MockImageRepository::new();
        mock.expect_local_digest()
            .returning(|_| Ok(Some("sha256:old".to_string())));
        mock.expect_registry_digest()
            .returning(|_| Ok(Some("sha256:new".to_string())));
        mock.expect_pull_images().returning(|_| {
            Ok(ImagePullSubscription::completed(Ok(ImagePullOutcome {
                pulled: vec!["nginx:latest".to_string()],
                up_to_date: vec![],
            })))
        });
        let service = ImageService::new(Arc::new(mock));
        let mut subscription = service
            .pull_images(&["nginx:latest".to_string()])
            .await
            .unwrap();
        match subscription.try_recv().unwrap() {
            ImagePullEvent::Completed(result) => {
                assert_eq!(result.unwrap().pulled, vec!["nginx:latest".to_string()]);
            }
            _ => panic!("expected completed event"),
        }
    }

    #[tokio::test]
    async fn test_pull_images_pulls_when_digests_unknown() {
        let mut mock = MockImageRepository::new();
        mock.expect_local_digest().returning(|_| Ok(None));
        mock.expect_registry_digest().returning(|_| Ok(None));
        mock.expect_pull_images().returning(|_| {
            Ok(ImagePullSubscription::completed(Ok(ImagePullOutcome {
                pulled: vec!["nginx:latest".to_string()],
                up_to_date: vec![],
            })))
        });
        let service = ImageService::new(Arc::new(mock));
        let mut subscription = service
            .pull_images(&["nginx:latest".to_string()])
            .await
            .unwrap();
        match subscription.try_recv().unwrap() {
            ImagePullEvent::Completed(result) => {
                assert_eq!(result.unwrap().pulled, vec!["nginx:latest".to_string()]);
            }
            _ => panic!("expected completed event"),
        }
    }

    #[tokio::test]
    async fn test_pull_images_pulls_when_image_missing_locally() {
        let mut mock = MockImageRepository::new();
        mock.expect_local_digest().returning(|_| Ok(None));
        mock.expect_pull_images().returning(|_| {
            Ok(ImagePullSubscription::completed(Ok(ImagePullOutcome {
                pulled: vec!["nginx:latest".to_string()],
                up_to_date: vec![],
            })))
        });
        let service = ImageService::new(Arc::new(mock));
        let mut subscription = service
            .pull_images(&["nginx:latest".to_string()])
            .await
            .unwrap();
        match subscription.try_recv().unwrap() {
            ImagePullEvent::Completed(result) => {
                assert_eq!(result.unwrap().pulled, vec!["nginx:latest".to_string()]);
            }
            _ => panic!("expected completed event"),
        }
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
