use crate::application::error::AppError;
use crate::application::PruneResultDto;
use crate::domain::Image;
use async_trait::async_trait;

#[async_trait]
pub trait ImageRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Image>, AppError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Image>, AppError>;
    async fn delete(&self, id: &str, force: bool) -> Result<(), AppError>;
    async fn prune(&self) -> Result<PruneResultDto, AppError>;
}
