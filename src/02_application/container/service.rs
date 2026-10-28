use crate::application::container::archive::extract_archive;
use crate::application::container::dto::{
    ContainerDto, ContainerLogsDto, ContainerStatsSubscription, CreateContainerRequest,
    FileContentDto, FileListingDto,
};
use crate::application::container::log_parser;
use crate::application::container::mapper::ContainerMapper;
use crate::application::container::traits::ContainerRepository;
use crate::application::error::AppError;
use crate::application::shared::PruneResultDto;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Maximum number of bytes read when viewing a file inside a container.
pub const FILE_VIEW_MAX_BYTES: u64 = 1024 * 1024;

/// Number of lines the file view scrolls on a single PageDown key press.
pub const FILE_VIEW_PAGE_SCROLL: usize = 10;

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
        let raw_logs = self.repository.get_logs(id, tail).await?;
        Ok(ContainerLogsDto {
            container_id: id.to_string(),
            container_name: name.to_string(),
            lines: log_parser::parse_log_lines(&raw_logs),
        })
    }

    pub async fn export_logs_to(
        &self,
        id: &str,
        name: &str,
        export_dir: &Path,
    ) -> Result<PathBuf, AppError> {
        let logs = self.repository.export_logs(id).await?;
        let path = Self::export_file_path(name, export_dir);
        tokio::fs::write(&path, logs)
            .await
            .map_err(|e| AppError::operation_failed(format!("cannot write {path:?}: {e}")))?;
        Ok(path)
    }

    pub async fn export_logs(&self, id: &str, name: &str) -> Result<PathBuf, AppError> {
        let export_dir = std::env::current_dir()
            .map_err(|e| AppError::operation_failed(format!("cannot resolve current dir: {e}")))?;
        self.export_logs_to(id, name, &export_dir).await
    }

    pub fn export_file_path(name: &str, export_dir: &Path) -> PathBuf {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        export_dir.join(format!("{name}_{timestamp}.log"))
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

    pub async fn create_container(
        &self,
        request: &CreateContainerRequest,
    ) -> Result<String, AppError> {
        self.repository.create(request).await
    }
    pub async fn prune_containers(&self) -> Result<PruneResultDto, AppError> {
        self.repository.prune().await
    }

    pub async fn subscribe_stats(
        &self,
        container_ids: Vec<String>,
    ) -> Result<ContainerStatsSubscription, AppError> {
        self.repository.subscribe_stats(container_ids).await
    }

    pub async fn list_container_files(
        &self,
        id: &str,
        name: &str,
        path: &str,
    ) -> Result<FileListingDto, AppError> {
        let listing = self.repository.list_files(id, path).await?;
        Ok(ContainerMapper::file_listing_to_dto(&listing, id, name))
    }

    pub async fn read_container_file(
        &self,
        id: &str,
        path: &str,
        max_bytes: u64,
    ) -> Result<FileContentDto, AppError> {
        let (data, total_size) = self.repository.read_file(id, path, max_bytes).await?;
        Ok(FileContentDto {
            truncated: total_size > max_bytes,
            data,
            total_size,
        })
    }

    pub async fn copy_to_host(
        &self,
        id: &str,
        path: &str,
        dest_dir: &Path,
    ) -> Result<PathBuf, AppError> {
        let archive = self.repository.download_archive(id, path).await?;
        extract_archive(&archive, dest_dir)
    }

    pub async fn copy_to_current_dir(&self, id: &str, path: &str) -> Result<PathBuf, AppError> {
        let dest_dir = std::env::current_dir()
            .map_err(|e| AppError::operation_failed(format!("cannot resolve current dir: {e}")))?;
        self.copy_to_host(id, path, &dest_dir).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::container::traits::MockContainerRepository;
    use crate::domain::container::{Container, ContainerId, ContainerState};
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
            .returning(|_, _| Ok("2026-10-26T10:30:00Z line1\nline2\n".to_string()));

        let service = ContainerService::new(Arc::new(mock));
        let result = service.get_logs("abc123", "web", Some(100)).await.unwrap();
        assert_eq!(result.container_id, "abc123");
        assert_eq!(result.container_name, "web");
        assert_eq!(result.lines.len(), 2);
        assert!(result.lines[0].timestamp.is_some());
        assert_eq!(result.lines[0].message, "line1");
        assert_eq!(result.lines[1].timestamp, None);
        assert_eq!(result.lines[1].message, "line2");
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

    #[tokio::test]
    async fn test_subscribe_stats_delegates() {
        let mut mock = MockContainerRepository::new();
        mock.expect_subscribe_stats()
            .withf(|ids| ids == &vec!["abc123".to_string()])
            .returning(|_| Ok(ContainerStatsSubscription::empty()));

        let service = ContainerService::new(Arc::new(mock));
        assert!(service
            .subscribe_stats(vec!["abc123".to_string()])
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_export_logs_writes_file_and_returns_path() {
        let dir = std::env::temp_dir().join(format!("rusty_console_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let mut mock = MockContainerRepository::new();
        mock.expect_export_logs()
            .withf(|id| id == "abc123")
            .returning(|_| Ok("2026-10-26T10:30:00Z line1\nline2\n".to_string()));
        let service = ContainerService::new(Arc::new(mock));

        let result = service.export_logs_to("abc123", "web", &dir).await.unwrap();
        assert!(result
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("web_"));
        assert!(result
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with(".log"));
        let content = std::fs::read_to_string(&result).unwrap();
        assert!(content.contains("line1"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn test_export_logs_propagates_repository_error() {
        let mut mock = MockContainerRepository::new();
        mock.expect_export_logs()
            .returning(|_| Err(AppError::repository("docker down")));
        let service = ContainerService::new(Arc::new(mock));
        let err = service.export_logs("abc123", "web").await.unwrap_err();
        assert!(err.to_string().contains("docker down"));
    }

    #[tokio::test]
    async fn test_create_container_delegates_and_returns_id() {
        let mut mock = MockContainerRepository::new();
        mock.expect_create()
            .withf(|request| request.name == "web" && request.image == "nginx:latest")
            .returning(|_| Ok("abc123".to_string()));
        let service = ContainerService::new(Arc::new(mock));
        let request = CreateContainerRequest::new("web", "nginx:latest", &[]);
        let id = service.create_container(&request).await.unwrap();
        assert_eq!(id, "abc123");
    }

    #[tokio::test]
    async fn test_create_container_propagates_repository_error() {
        let mut mock = MockContainerRepository::new();
        mock.expect_create()
            .returning(|_| Err(AppError::repository("docker down")));
        let service = ContainerService::new(Arc::new(mock));
        let request = CreateContainerRequest::new("web", "nginx:latest", &[]);
        let err = service.create_container(&request).await.unwrap_err();
        assert!(err.to_string().contains("docker down"));
    }

    #[test]
    fn test_export_file_path_format() {
        let path = ContainerService::export_file_path("my-web", Path::new("/tmp"));
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        assert!(name.starts_with("my-web_"));
        assert!(name.ends_with(".log"));
        let timestamp_part = name.trim_start_matches("my-web_").trim_end_matches(".log");
        assert_eq!(timestamp_part.len(), 15);
    }

    fn make_file_listing(path: &str) -> crate::domain::container::FileListing {
        use crate::domain::container::{FileEntry, FileKind, FilePermissions};
        crate::domain::container::FileListing::new(
            path,
            vec![FileEntry::new(
                "app.conf",
                "/app.conf",
                FileKind::File,
                3,
                None,
                FilePermissions::new(0o100644),
            )
            .unwrap()],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_list_container_files_returns_dto() {
        let mut mock = MockContainerRepository::new();
        mock.expect_list_files()
            .withf(|id, path| id == "abc123" && path == "/etc")
            .returning(|_, _| Ok(make_file_listing("/etc")));

        let service = ContainerService::new(Arc::new(mock));
        let dto = service
            .list_container_files("abc123", "web", "/etc")
            .await
            .unwrap();
        assert_eq!(dto.container_id, "abc123");
        assert_eq!(dto.container_name, "web");
        assert_eq!(dto.path, "/etc");
        assert_eq!(dto.entries.len(), 1);
        assert_eq!(dto.entries[0].name, "app.conf");
    }

    #[tokio::test]
    async fn test_list_container_files_propagates_error() {
        let mut mock = MockContainerRepository::new();
        mock.expect_list_files()
            .returning(|_, _| Err(AppError::repository("container stopped")));

        let service = ContainerService::new(Arc::new(mock));
        let err = service
            .list_container_files("abc123", "web", "/")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("container stopped"));
    }

    #[tokio::test]
    async fn test_read_container_file_marks_truncation() {
        let mut mock = MockContainerRepository::new();
        mock.expect_read_file()
            .withf(|id, path, max| id == "abc123" && path == "/var/log/app.log" && *max == 100)
            .returning(|_, _, _| Ok((vec![b'x'; 100], 5_000)));

        let service = ContainerService::new(Arc::new(mock));
        let content = service
            .read_container_file("abc123", "/var/log/app.log", 100)
            .await
            .unwrap();
        assert!(content.truncated);
        assert_eq!(content.total_size, 5_000);
        assert_eq!(content.data.len(), 100);
    }

    #[tokio::test]
    async fn test_read_container_file_not_truncated_when_within_limit() {
        let mut mock = MockContainerRepository::new();
        mock.expect_read_file()
            .returning(|_, _, _| Ok((b"hello".to_vec(), 5)));

        let service = ContainerService::new(Arc::new(mock));
        let content = service
            .read_container_file("abc123", "/f", 100)
            .await
            .unwrap();
        assert!(!content.truncated);
        assert_eq!(content.text(), Some("hello".to_string()));
    }

    #[tokio::test]
    async fn test_copy_to_host_extracts_archive_into_dest_dir() {
        let mut builder = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(5);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "app.conf", &b"hello"[..])
            .unwrap();
        let archive = builder.into_inner().unwrap();

        let dir =
            std::env::temp_dir().join(format!("rusty_console_copy_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let mut mock = MockContainerRepository::new();
        mock.expect_download_archive()
            .withf(|id, path| id == "abc123" && path == "/app.conf")
            .returning(move |_, _| Ok(archive.clone()));

        let service = ContainerService::new(Arc::new(mock));
        let result = service
            .copy_to_host("abc123", "/app.conf", &dir)
            .await
            .unwrap();
        assert_eq!(result, dir.join("app.conf"));
        assert_eq!(std::fs::read_to_string(&result).unwrap(), "hello");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn test_copy_to_host_propagates_download_error() {
        let mut mock = MockContainerRepository::new();
        mock.expect_download_archive()
            .returning(|_, _| Err(AppError::repository("no such file")));

        let service = ContainerService::new(Arc::new(mock));
        let err = service
            .copy_to_host("abc123", "/missing", Path::new("/tmp"))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("no such file"));
    }
}
