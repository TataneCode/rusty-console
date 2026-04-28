use crate::application::container::{
    ContainerDto, ContainerLogsDto, ContainerService, ContainerStatsSubscription,
};
use crate::application::error::AppError;
use crate::shared::PruneResultDto;

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

    pub async fn subscribe_stats(
        &self,
        container_ids: Vec<String>,
    ) -> Result<ContainerStatsSubscription, AppError> {
        self.service.subscribe_stats(container_ids).await
    }
}

#[cfg(test)]
mod tests {
    use super::ContainerActions;
    use crate::application::container::traits::MockContainerRepository;
    use crate::application::container::{ContainerService, ContainerStatsSubscription};
    use crate::domain::container::{Container, ContainerId, ContainerState};
    use crate::shared::PruneResultDto;
    use chrono::Utc;
    use std::sync::Arc;

    fn make_container() -> Container {
        Container::new(
            ContainerId::new("abc123").unwrap(),
            "web",
            "nginx:latest",
            ContainerState::Running,
            "Up 1 minute",
            Utc::now(),
        )
    }

    fn make_action(mock: MockContainerRepository) -> ContainerActions {
        ContainerActions::new(ContainerService::new(Arc::new(mock)))
    }

    #[tokio::test]
    async fn test_container_actions_delegate_all_operations() {
        let mut mock = MockContainerRepository::new();
        mock.expect_get_all()
            .returning(|| Ok(vec![make_container()]));
        mock.expect_get_logs()
            .returning(|_, _| Ok("line1\nline2".to_string()));
        mock.expect_get_by_id()
            .returning(|_| Ok(Some(make_container())));
        mock.expect_start().returning(|_| Ok(()));
        mock.expect_stop().returning(|_| Ok(()));
        mock.expect_delete().returning(|_, _| Ok(()));
        mock.expect_restart().returning(|_| Ok(()));
        mock.expect_pause().returning(|_| Ok(()));
        mock.expect_unpause().returning(|_| Ok(()));
        mock.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 2,
                space_freed: 1024,
            })
        });
        mock.expect_subscribe_stats()
            .returning(|_| Ok(ContainerStatsSubscription::empty()));

        let actions = make_action(mock);
        let containers = actions.load_containers().await.unwrap();
        let container = containers[0].clone();

        assert_eq!(containers.len(), 1);
        assert_eq!(
            actions.load_logs(&container, Some(10)).await.unwrap().logs,
            "line1\nline2"
        );
        assert!(actions
            .load_container_details(&container.id)
            .await
            .unwrap()
            .is_some());
        assert!(actions.start_container(&container.id).await.is_ok());
        assert!(actions.stop_container(&container.id).await.is_ok());
        assert!(actions.delete_container(&container.id, true).await.is_ok());
        assert!(actions.restart_container(&container.id).await.is_ok());
        assert!(actions.pause_container(&container.id).await.is_ok());
        assert!(actions.unpause_container(&container.id).await.is_ok());
        assert_eq!(actions.prune_containers().await.unwrap().deleted_count, 2);
        assert!(actions
            .subscribe_stats(vec![container.id.clone()])
            .await
            .is_ok());
    }
}
