#![allow(clippy::double_must_use)]
use crate::application::container::dto::{ContainerStatsSubscription, CreateContainerRequest};
use crate::application::error::AppError;
use crate::application::shared::PruneResultDto;
use crate::domain::container::{Container, FileListing};
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ContainerRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Container>, AppError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Container>, AppError>;
    async fn get_logs(&self, id: &str, tail: Option<usize>) -> Result<String, AppError>;
    async fn export_logs(&self, id: &str) -> Result<String, AppError>;
    async fn start(&self, id: &str) -> Result<(), AppError>;
    async fn create(&self, request: &CreateContainerRequest) -> Result<String, AppError>;
    async fn stop(&self, id: &str) -> Result<(), AppError>;
    async fn delete(&self, id: &str, force: bool) -> Result<(), AppError>;
    async fn restart(&self, id: &str) -> Result<(), AppError>;
    async fn pause(&self, id: &str) -> Result<(), AppError>;
    async fn unpause(&self, id: &str) -> Result<(), AppError>;
    async fn prune(&self) -> Result<PruneResultDto, AppError>;
    async fn subscribe_stats(
        &self,
        container_ids: Vec<String>,
    ) -> Result<ContainerStatsSubscription, AppError>;
    async fn list_files(&self, id: &str, path: &str) -> Result<FileListing, AppError>;
    /// Downloads up to `max_bytes` of the file; returns the bytes read and
    /// the real file size (which may exceed `max_bytes`).
    async fn read_file(
        &self,
        id: &str,
        path: &str,
        max_bytes: u64,
    ) -> Result<(Vec<u8>, u64), AppError>;
    async fn download_archive(&self, id: &str, path: &str) -> Result<Vec<u8>, AppError>;
}
