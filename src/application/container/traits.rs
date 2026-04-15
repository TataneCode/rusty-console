use crate::application::error::AppError;
use crate::application::PruneResultDto;
use crate::domain::Container;
use async_trait::async_trait;

#[async_trait]
pub trait ContainerRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Container>, AppError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Container>, AppError>;
    async fn get_logs(&self, id: &str, tail: Option<usize>) -> Result<String, AppError>;
    async fn start(&self, id: &str) -> Result<(), AppError>;
    async fn stop(&self, id: &str) -> Result<(), AppError>;
    async fn delete(&self, id: &str, force: bool) -> Result<(), AppError>;
    async fn restart(&self, id: &str) -> Result<(), AppError>;
    async fn pause(&self, id: &str) -> Result<(), AppError>;
    async fn unpause(&self, id: &str) -> Result<(), AppError>;
    async fn prune(&self) -> Result<PruneResultDto, AppError>;
}
