use crate::errors::AppError;
use crate::stack::application::{StackDto, StackService};

pub struct StackActions {
    service: StackService,
}

impl StackActions {
    pub fn new(service: StackService) -> Self {
        StackActions { service }
    }

    pub async fn load_stacks(&self) -> Result<Vec<StackDto>, AppError> {
        self.service.get_all_stacks().await
    }

    pub async fn start_all(&self, container_ids: &[String]) -> Result<(), AppError> {
        self.service.start_all(container_ids).await
    }

    pub async fn stop_all(&self, container_ids: &[String]) -> Result<(), AppError> {
        self.service.stop_all(container_ids).await
    }

    pub async fn remove_all(&self, container_ids: &[String]) -> Result<(), AppError> {
        self.service.remove_all(container_ids).await
    }
}
