use crate::application::error::AppError;
use crate::domain::stack::Stack;
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait StackRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Stack>, AppError>;
    async fn start_all(&self, container_ids: &[String]) -> Result<(), AppError>;
    async fn stop_all(&self, container_ids: &[String]) -> Result<(), AppError>;
    async fn remove_all(&self, container_ids: &[String]) -> Result<(), AppError>;
    async fn pull_images(&self, image_refs: &[String]) -> Result<(), AppError>;
}
