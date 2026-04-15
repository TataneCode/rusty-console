use crate::application::AppError;
use crate::application::PruneResultDto;
use crate::application::{ContainerDto, ContainerLogsDto, ContainerService};

pub struct ContainerActions {
    service: ContainerService,
}

impl ContainerActions {
    pub fn new(service: ContainerService) -> Self {
        ContainerActions { service }
    }

    pub async fn load_containers(&self) -> Result<Vec<ContainerDto>, AppError> {
        self.service.get_all_containers().await
    }

    pub async fn load_logs(
        &self,
        container: &ContainerDto,
        tail: Option<usize>,
    ) -> Result<ContainerLogsDto, AppError> {
        self.service
            .get_logs(&container.id, &container.name, tail)
            .await
    }

    pub async fn load_container_details(&self, id: &str) -> Result<Option<ContainerDto>, AppError> {
        self.service.get_container_by_id(id).await
    }

    pub async fn start_container(&self, id: &str) -> Result<(), AppError> {
        self.service.start_container(id).await
    }

    pub async fn stop_container(&self, id: &str) -> Result<(), AppError> {
        self.service.stop_container(id).await
    }

    pub async fn delete_container(&self, id: &str, force: bool) -> Result<(), AppError> {
        self.service.delete_container(id, force).await
    }

    pub async fn restart_container(&self, id: &str) -> Result<(), AppError> {
        self.service.restart_container(id).await
    }

    pub async fn pause_container(&self, id: &str) -> Result<(), AppError> {
        self.service.pause_container(id).await
    }

    pub async fn unpause_container(&self, id: &str) -> Result<(), AppError> {
        self.service.unpause_container(id).await
    }

    pub async fn prune_containers(&self) -> Result<PruneResultDto, AppError> {
        self.service.prune_containers().await
    }
}
