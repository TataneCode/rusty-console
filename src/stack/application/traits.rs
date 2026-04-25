use crate::errors::AppError;
use crate::stack::domain::Stack;
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait StackRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Stack>, AppError>;
    async fn start_all(&self, container_ids: &[String]) -> Result<(), AppError>;
    async fn stop_all(&self, container_ids: &[String]) -> Result<(), AppError>;
    async fn remove_all(&self, container_ids: &[String]) -> Result<(), AppError>;
}
