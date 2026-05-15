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

    pub async fn pull_images(&self, image_refs: &[String]) -> Result<(), AppError> {
        self.service.pull_images(image_refs).await
    }
}

#[cfg(test)]
mod tests {
    use super::StackActions;
    use crate::application::stack::traits::MockStackRepository;
    use crate::application::stack::StackService;
    use crate::domain::stack::{Stack, StackName};
    use std::sync::Arc;

    fn make_stack() -> Stack {
        Stack::new(StackName::new("compose-app").unwrap(), vec![])
    }

    #[tokio::test]
    async fn test_stack_actions_delegate_all_operations() {
        let mut mock = MockStackRepository::new();
        mock.expect_get_all().returning(|| Ok(vec![make_stack()]));
        mock.expect_start_all().returning(|_| Ok(()));
        mock.expect_stop_all().returning(|_| Ok(()));
        mock.expect_remove_all().returning(|_| Ok(()));
        mock.expect_pull_images().returning(|_| Ok(()));

        let actions = StackActions::new(StackService::new(Arc::new(mock)));
        let ids = vec!["1".to_string(), "2".to_string()];
        let images = vec!["nginx:latest".to_string()];

        assert_eq!(actions.load_stacks().await.unwrap().len(), 1);
        assert!(actions.start_all(&ids).await.is_ok());
        assert!(actions.stop_all(&ids).await.is_ok());
        assert!(actions.remove_all(&ids).await.is_ok());
        assert!(actions.pull_images(&images).await.is_ok());
    }
}
