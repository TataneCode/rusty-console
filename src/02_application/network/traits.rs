use crate::application::error::AppError;
use crate::domain::network::Network;
use async_trait::async_trait;

use crate::application::shared::PruneResultDto;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NetworkRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Network>, AppError>;
    async fn get_by_name(&self, name: &str) -> Result<Option<Network>, AppError>;
    async fn inspect(&self, name: &str) -> Result<Network, AppError>;
    async fn delete(&self, name: &str) -> Result<(), AppError>;
    async fn prune(&self) -> Result<PruneResultDto, AppError>;
    async fn disconnect(
        &self,
        network: &str,
        container: &str,
        force: bool,
    ) -> Result<(), AppError>;
}
