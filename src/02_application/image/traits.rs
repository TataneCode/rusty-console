use crate::errors::AppError;
use crate::image::domain::Image;
use crate::shared::PruneResultDto;
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ImageRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Image>, AppError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Image>, AppError>;
    async fn delete(&self, id: &str, force: bool) -> Result<(), AppError>;
    async fn prune(&self) -> Result<PruneResultDto, AppError>;
}
