use crate::application::error::AppError;
use crate::application::stack::{StackDto, StackService};

#[derive(Clone)]
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

#[cfg(test)]
mod tests {
    use super::StackActions;
    use crate::application::stack::traits::MockStackRepository;
    use crate::application::stack::StackService;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_load_stacks_delegates_to_service() {
        let mut mock = MockStackRepository::new();
        mock.expect_get_all().returning(|| Ok(vec![]));
        let actions = StackActions::new(StackService::new(Arc::new(mock)));
        assert!(actions.load_stacks().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_start_stop_delegates_to_service() {
        let mut mock = MockStackRepository::new();
        mock.expect_start_all().returning(|_| Ok(()));
        mock.expect_stop_all().returning(|_| Ok(()));
        let actions = StackActions::new(StackService::new(Arc::new(mock)));
        let ids = vec!["abc123".to_string()];
        assert!(actions.start_all(&ids).await.is_ok());
        assert!(actions.stop_all(&ids).await.is_ok());
    }
}
