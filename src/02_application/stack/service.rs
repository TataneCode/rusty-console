use crate::application::error::AppError;
use crate::application::stack::dto::StackDto;
use crate::application::stack::mapper::StackMapper;
use crate::application::stack::traits::StackRepository;
use std::sync::Arc;

pub struct StackService {
    repository: Arc<dyn StackRepository>,
}

impl StackService {
    pub fn new(repository: Arc<dyn StackRepository>) -> Self {
        StackService { repository }
    }

    pub async fn get_all_stacks(&self) -> Result<Vec<StackDto>, AppError> {
        let stacks = self.repository.get_all().await?;
        Ok(StackMapper::to_dto_list(&stacks))
    }

    pub async fn start_all(&self, container_ids: &[String]) -> Result<(), AppError> {
        self.repository.start_all(container_ids).await
    }

    pub async fn stop_all(&self, container_ids: &[String]) -> Result<(), AppError> {
        self.repository.stop_all(container_ids).await
    }
    pub async fn remove_all(&self, container_ids: &[String]) -> Result<(), AppError> {
        self.repository.remove_all(container_ids).await
    }

    pub async fn pull_images(&self, image_refs: &[String]) -> Result<(), AppError> {
        self.repository.pull_images(image_refs).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::stack::traits::MockStackRepository;
    use crate::domain::stack::{Stack, StackName};
    use mockall::predicate::eq;

    fn make_stack(name: &str) -> Stack {
        Stack::new(StackName::new(name).unwrap(), vec![])
    }

    #[tokio::test]
    async fn test_list_stacks_returns_dtos() {
        let mut mock = MockStackRepository::new();
        mock.expect_get_all()
            .returning(|| Ok(vec![make_stack("app-a"), make_stack("app-b")]));

        let service = StackService::new(Arc::new(mock));
        let result = service.get_all_stacks().await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "app-a");
        assert_eq!(result[1].name, "app-b");
    }

    #[tokio::test]
    async fn test_start_all_delegates_to_repository() {
        let ids = vec!["id1".to_string(), "id2".to_string()];
        let mut mock = MockStackRepository::new();
        mock.expect_start_all()
            .with(eq(ids.clone()))
            .returning(|_| Ok(()));

        let service = StackService::new(Arc::new(mock));
        assert!(service.start_all(&ids).await.is_ok());
    }

    #[tokio::test]
    async fn test_stop_all_delegates_to_repository() {
        let ids = vec!["id1".to_string()];
        let mut mock = MockStackRepository::new();
        mock.expect_stop_all()
            .with(eq(ids.clone()))
            .returning(|_| Ok(()));

        let service = StackService::new(Arc::new(mock));
        assert!(service.stop_all(&ids).await.is_ok());
    }

    #[tokio::test]
    async fn test_list_stacks_empty() {
        let mut mock = MockStackRepository::new();
        mock.expect_get_all().returning(|| Ok(vec![]));

        let service = StackService::new(Arc::new(mock));
        let result = service.get_all_stacks().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_remove_all_delegates_to_repository() {
        let ids = vec!["id1".to_string(), "id2".to_string()];
        let mut mock = MockStackRepository::new();
        mock.expect_remove_all()
            .with(eq(ids.clone()))
            .returning(|_| Ok(()));

        let service = StackService::new(Arc::new(mock));
        assert!(service.remove_all(&ids).await.is_ok());
    }

    #[tokio::test]
    async fn test_pull_images_delegates_to_repository() {
        let images = vec!["nginx:latest".to_string(), "postgres:16".to_string()];
        let expected = images.clone();
        let mut mock = MockStackRepository::new();
        mock.expect_pull_images()
            .withf(move |image_refs| image_refs == expected.as_slice())
            .returning(|_| Ok(()));

        let service = StackService::new(Arc::new(mock));
        assert!(service.pull_images(&images).await.is_ok());
    }
}
