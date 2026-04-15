use crate::application::container::dto::{ContainerDto, ContainerLogsDto};
use crate::application::container::mapper::ContainerMapper;
use crate::application::container::traits::ContainerRepository;
use crate::application::error::AppError;
use std::sync::Arc;

pub struct ContainerService {
    repository: Arc<dyn ContainerRepository>,
}

impl ContainerService {
    pub fn new(repository: Arc<dyn ContainerRepository>) -> Self {
        ContainerService { repository }
    }

    pub async fn get_all_containers(&self) -> Result<Vec<ContainerDto>, AppError> {
        let containers = self.repository.get_all().await?;
        Ok(ContainerMapper::to_dto_list(&containers))
    }

    pub async fn get_container_by_id(&self, id: &str) -> Result<Option<ContainerDto>, AppError> {
        let container = self.repository.get_by_id(id).await?;
        Ok(container.map(|c| ContainerMapper::to_dto(&c)))
    }

    pub async fn get_logs(
        &self,
        id: &str,
        name: &str,
        tail: Option<usize>,
    ) -> Result<ContainerLogsDto, AppError> {
        let logs = self.repository.get_logs(id, tail).await?;
        Ok(ContainerLogsDto {
            container_id: id.to_string(),
            container_name: name.to_string(),
            logs,
        })
    }

    pub async fn start_container(&self, id: &str) -> Result<(), AppError> {
        self.repository.start(id).await
    }

    pub async fn stop_container(&self, id: &str) -> Result<(), AppError> {
        self.repository.stop(id).await
    }

    pub async fn delete_container(&self, id: &str, force: bool) -> Result<(), AppError> {
        self.repository.delete(id, force).await
    }

    pub async fn restart_container(&self, id: &str) -> Result<(), AppError> {
        self.repository.restart(id).await
    }

    pub async fn pause_container(&self, id: &str) -> Result<(), AppError> {
        self.repository.pause(id).await
    }

    pub async fn unpause_container(&self, id: &str) -> Result<(), AppError> {
        self.repository.unpause(id).await
    }
}
