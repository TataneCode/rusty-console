use crate::container::application::dto::{ContainerDto, ContainerLogsDto};
use crate::container::application::mapper::ContainerMapper;
use crate::container::application::traits::ContainerRepository;
use crate::errors::AppError;
use crate::shared::PruneResultDto;
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

    pub async fn prune_containers(&self) -> Result<PruneResultDto, AppError> {
        self.repository.prune().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::application::traits::MockContainerRepository;
    use crate::container::domain::{Container, ContainerId, ContainerState};
    use chrono::Utc;
    use std::sync::Arc;

    fn make_container(id: &str, name: &str, state: ContainerState) -> Container {
        Container::new(
            ContainerId::new(id).unwrap(),
            name,
            "nginx:latest",
            state,
            "Up 5 minutes",
            Utc::now(),
        )
    }

    #[tokio::test]
    async fn test_get_all_containers_returns_mapped_dtos() {
        let mut mock = MockContainerRepository::new();
        mock.expect_get_all().returning(|| {
            Ok(vec![
                make_container("abc123", "/web", ContainerState::Running),
                make_container("def456", "/db", ContainerState::Stopped),
            ])
        });

        let service = ContainerService::new(Arc::new(mock));
        let result = service.get_all_containers().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "abc123");
        assert_eq!(result[0].name, "web");
        assert_eq!(result[1].id, "def456");
        assert_eq!(result[1].name, "db");
    }

    #[tokio::test]
    async fn test_get_all_containers_propagates_error() {
        let mut mock = MockContainerRepository::new();
        mock.expect_get_all()
            .returning(|| Err(AppError::repository("connection failed")));

        let service = ContainerService::new(Arc::new(mock));
        let result = service.get_all_containers().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("connection failed"));
    }

    #[tokio::test]
    async fn test_get_container_by_id_found() {
        let mut mock = MockContainerRepository::new();
        mock.expect_get_by_id()
            .withf(|id| id == "abc123")
            .returning(|_| {
                Ok(Some(make_container(
                    "abc123",
                    "/web",
                    ContainerState::Running,
                )))
            });

        let service = ContainerService::new(Arc::new(mock));
        let result = service.get_container_by_id("abc123").await.unwrap();
        assert!(result.is_some());
        let dto = result.unwrap();
        assert_eq!(dto.id, "abc123");
        assert_eq!(dto.name, "web");
        assert!(dto.can_stop);
        assert!(!dto.can_start);
    }

    #[tokio::test]
    async fn test_get_container_by_id_not_found() {
        let mut mock = MockContainerRepository::new();
        mock.expect_get_by_id().returning(|_| Ok(None));

        let service = ContainerService::new(Arc::new(mock));
        let result = service.get_container_by_id("missing").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_logs_returns_dto() {
        let mut mock = MockContainerRepository::new();
        mock.expect_get_logs()
            .withf(|id, tail| id == "abc123" && *tail == Some(100))
            .returning(|_, _| Ok("line1\nline2".to_string()));

        let service = ContainerService::new(Arc::new(mock));
        let result = service.get_logs("abc123", "web", Some(100)).await.unwrap();
        assert_eq!(result.container_id, "abc123");
        assert_eq!(result.container_name, "web");
        assert_eq!(result.logs, "line1\nline2");
    }

    #[tokio::test]
    async fn test_start_container_delegates() {
        let mut mock = MockContainerRepository::new();
        mock.expect_start()
            .withf(|id| id == "abc123")
            .returning(|_| Ok(()));

        let service = ContainerService::new(Arc::new(mock));
        assert!(service.start_container("abc123").await.is_ok());
    }

    #[tokio::test]
    async fn test_stop_container_delegates() {
        let mut mock = MockContainerRepository::new();
        mock.expect_stop()
            .withf(|id| id == "abc123")
            .returning(|_| Ok(()));

        let service = ContainerService::new(Arc::new(mock));
        assert!(service.stop_container("abc123").await.is_ok());
    }

    #[tokio::test]
    async fn test_delete_container_delegates() {
        let mut mock = MockContainerRepository::new();
        mock.expect_delete()
            .withf(|id, force| id == "abc123" && *force)
            .returning(|_, _| Ok(()));

        let service = ContainerService::new(Arc::new(mock));
        assert!(service.delete_container("abc123", true).await.is_ok());
    }

    #[tokio::test]
    async fn test_restart_container_delegates() {
        let mut mock = MockContainerRepository::new();
        mock.expect_restart()
            .withf(|id| id == "abc123")
            .returning(|_| Ok(()));

        let service = ContainerService::new(Arc::new(mock));
        assert!(service.restart_container("abc123").await.is_ok());
    }

    #[tokio::test]
    async fn test_prune_containers_returns_result() {
        let mut mock = MockContainerRepository::new();
        mock.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 3,
                space_freed: 1024,
            })
        });

        let service = ContainerService::new(Arc::new(mock));
        let result = service.prune_containers().await.unwrap();
        assert_eq!(result.deleted_count, 3);
        assert_eq!(result.space_freed, 1024);
    }
}
