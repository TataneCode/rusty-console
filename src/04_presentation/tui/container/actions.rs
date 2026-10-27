use crate::application::container::{
    ContainerDto, ContainerLogsDto, ContainerService, ContainerStatsSubscription,
    CreateContainerRequest, FileContentDto, FileListingDto,
};
use crate::application::error::AppError;
use crate::application::shared::PruneResultDto;
use std::path::PathBuf;

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

    pub async fn load_logs_by(
        &self,
        container_id: &str,
        container_name: &str,
        tail: Option<usize>,
    ) -> Result<ContainerLogsDto, AppError> {
        self.service
            .get_logs(container_id, container_name, tail)
            .await
    }

    pub async fn export_logs_by(
        &self,
        container_id: &str,
        container_name: &str,
    ) -> Result<PathBuf, AppError> {
        self.service.export_logs(container_id, container_name).await
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

    pub async fn create_container(
        &self,
        request: &CreateContainerRequest,
    ) -> Result<String, AppError> {
        self.service.create_container(request).await
    }

    pub async fn subscribe_stats(
        &self,
        container_ids: Vec<String>,
    ) -> Result<ContainerStatsSubscription, AppError> {
        self.service.subscribe_stats(container_ids).await
    }

    pub async fn list_files(
        &self,
        container_id: &str,
        container_name: &str,
        path: &str,
    ) -> Result<FileListingDto, AppError> {
        self.service
            .list_container_files(container_id, container_name, path)
            .await
    }

    pub async fn read_file(
        &self,
        container_id: &str,
        path: &str,
        max_bytes: u64,
    ) -> Result<FileContentDto, AppError> {
        self.service
            .read_container_file(container_id, path, max_bytes)
            .await
    }

    pub async fn copy_to_host(&self, container_id: &str, path: &str) -> Result<PathBuf, AppError> {
        self.service.copy_to_current_dir(container_id, path).await
    }
}

#[cfg(test)]
mod tests {
    use super::ContainerActions;
    use crate::application::container::traits::MockContainerRepository;
    use crate::application::container::{ContainerService, ContainerStatsSubscription};
    use crate::application::shared::PruneResultDto;
    use crate::domain::container::{Container, ContainerId, ContainerState};
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
        let logs = actions.load_logs(&container, Some(10)).await.unwrap();
        assert_eq!(logs.lines.len(), 2);
        assert_eq!(logs.lines[0].message, "line1");
        assert_eq!(logs.lines[1].message, "line2");
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

    #[tokio::test]
    async fn test_container_actions_delegate_file_operations() {
        use crate::application::container::{FileEntryDto, FileListingDto};
        use crate::domain::container::FileKind;

        let mut mock = MockContainerRepository::new();
        mock.expect_list_files().returning(|_, _| {
            use crate::domain::container::{FileEntry, FileListing, FilePermissions};
            Ok(FileListing::new(
                "/etc",
                vec![FileEntry::new(
                    "nginx",
                    "/etc/nginx",
                    FileKind::Directory,
                    0,
                    None,
                    FilePermissions::new(0o040755),
                )
                .unwrap()],
            )
            .unwrap())
        });
        mock.expect_read_file()
            .returning(|_, _, _| Ok((b"hello".to_vec(), 5)));
        mock.expect_download_archive()
            .returning(|_, _| Ok(vec![0x1f, 0x8b]));

        let actions = make_action(mock);
        let listing: FileListingDto = actions.list_files("abc123", "web", "/etc").await.unwrap();
        assert_eq!(listing.path, "/etc");
        assert_eq!(listing.entries.len(), 1);
        assert_eq!(
            listing.entries[0],
            FileEntryDto {
                name: "nginx".to_string(),
                path: "/etc/nginx".to_string(),
                kind: FileKind::Directory,
                size: 0,
                modified: None,
                permissions: "drwxr-xr-x".to_string(),
                link_target: None,
            }
        );

        let content = actions
            .read_file("abc123", "/etc/app.conf", 1024)
            .await
            .unwrap();
        assert_eq!(content.text(), Some("hello".to_string()));
        assert!(!content.truncated);

        // The archive fixture is not a valid tar: copy must surface the error.
        assert!(actions.copy_to_host("abc123", "/etc/nginx").await.is_err());
    }
}
