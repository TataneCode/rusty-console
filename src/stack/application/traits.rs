use crate::errors::AppError;
use crate::stack::domain::Stack;
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait StackRepository: Send + Sync {
    async fn list_stacks(&self) -> Result<Vec<Stack>, AppError>;
    async fn start_all(&self, container_ids: &[&str]) -> Result<(), AppError>;
    async fn stop_all(&self, container_ids: &[&str]) -> Result<(), AppError>;
}
