use crate::application::error::AppError;
use crate::application::PruneResultDto;
use crate::domain::Volume;
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait VolumeRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Volume>, AppError>;
    async fn get_by_name(&self, name: &str) -> Result<Option<Volume>, AppError>;
    async fn delete(&self, name: &str) -> Result<(), AppError>;
    async fn prune(&self) -> Result<PruneResultDto, AppError>;
}
