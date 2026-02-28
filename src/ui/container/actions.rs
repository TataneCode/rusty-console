use crate::application::{ContainerDto, ContainerLogsDto, ContainerService};
use crate::application::AppError;

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

    pub async fn start_container(&self, id: &str) -> Result<(), AppError> {
        self.service.start_container(id).await
    }

    pub async fn stop_container(&self, id: &str) -> Result<(), AppError> {
        self.service.stop_container(id).await
    }

    pub async fn delete_container(&self, id: &str, force: bool) -> Result<(), AppError> {
        self.service.delete_container(id, force).await
    }
}
