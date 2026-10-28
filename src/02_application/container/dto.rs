use crate::application::error::AppError;
use crate::domain::container::{ContainerState, FileKind};
use crate::domain::shared::ByteSize;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct ContainerDto {
    pub id: String,
    pub short_id: String,
    pub name: String,
    pub image: String,
    pub state: ContainerState,
    pub status: String,
    pub created: String,
    pub ports: String,
    pub networks: String,
    pub can_start: bool,
    pub can_stop: bool,
    pub can_delete: bool,
    pub can_restart: bool,
    pub can_pause: bool,
    pub can_unpause: bool,
    pub env_vars: Vec<String>,
    pub runtime_stats: Option<ContainerRuntimeStatsDto>,
}

impl ContainerDto {
    pub fn state_display(&self) -> &'static str {
        match self.state {
            ContainerState::Running => "Running",
            ContainerState::Paused => "Paused",
            ContainerState::Stopped => "Stopped",
            ContainerState::Exited => "Exited",
            ContainerState::Dead => "Dead",
            ContainerState::Created => "Created",
            ContainerState::Removing => "Removing",
            ContainerState::Restarting => "Restarting",
        }
    }

    pub fn cpu_display(&self) -> String {
        self.runtime_stats
            .as_ref()
            .map(ContainerRuntimeStatsDto::cpu_display)
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn memory_display(&self) -> String {
        self.runtime_stats
            .as_ref()
            .map(ContainerRuntimeStatsDto::memory_list_display)
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn network_io_display(&self) -> String {
        self.runtime_stats
            .as_ref()
            .map(ContainerRuntimeStatsDto::network_io_display)
            .unwrap_or_else(|| "N/A".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ContainerLogsDto {
    pub container_id: String,
    pub container_name: String,
    pub lines: Vec<LogLineDto>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogLineDto {
    pub timestamp: Option<DateTime<Utc>>,
    pub message: String,
}

impl LogLineDto {
    pub fn timestamp_display(&self) -> String {
        self.timestamp
            .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default()
    }

    pub fn display_line(&self) -> String {
        let timestamp = self.timestamp_display();
        if timestamp.is_empty() {
            self.message.clone()
        } else {
            format!("{timestamp} {}", self.message)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerRuntimeStatsDto {
    pub cpu_percent: f64,
    pub memory_usage: ByteSize,
    pub memory_limit: ByteSize,
    pub memory_percent: f64,
    pub network_rx: ByteSize,
    pub network_tx: ByteSize,
}

impl ContainerRuntimeStatsDto {
    pub fn cpu_display(&self) -> String {
        format!("{:.1}%", self.cpu_percent)
    }

    pub fn memory_list_display(&self) -> String {
        if self.memory_limit.bytes() > 0 {
            format!(
                "{} ({:.0}%)",
                self.memory_usage.human_readable(),
                self.memory_percent
            )
        } else {
            self.memory_usage.human_readable()
        }
    }

    pub fn memory_details_display(&self) -> String {
        if self.memory_limit.bytes() > 0 {
            format!(
                "{} / {} ({:.1}%)",
                self.memory_usage.human_readable(),
                self.memory_limit.human_readable(),
                self.memory_percent
            )
        } else {
            self.memory_usage.human_readable()
        }
    }

    pub fn network_io_display(&self) -> String {
        format!(
            "RX {} / TX {}",
            self.network_rx.human_readable(),
            self.network_tx.human_readable()
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainerStatsUpdate {
    pub container_id: String,
    pub stats: ContainerRuntimeStatsDto,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerStatsEvent {
    Update(ContainerStatsUpdate),
    Error {
        container_id: String,
        message: String,
    },
}

#[derive(Debug)]
pub struct ContainerStatsSubscription {
    receiver: mpsc::Receiver<ContainerStatsEvent>,
    tasks: Vec<JoinHandle<()>>,
}

impl ContainerStatsSubscription {
    pub fn new(receiver: mpsc::Receiver<ContainerStatsEvent>, tasks: Vec<JoinHandle<()>>) -> Self {
        ContainerStatsSubscription { receiver, tasks }
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        let (_sender, receiver) = mpsc::channel(1);
        ContainerStatsSubscription::new(receiver, Vec::new())
    }

    pub fn try_recv(&mut self) -> Result<ContainerStatsEvent, mpsc::error::TryRecvError> {
        self.receiver.try_recv()
    }

    pub fn abort(&mut self) {
        for task in self.tasks.drain(..) {
            task.abort();
        }
    }
}

impl Drop for ContainerStatsSubscription {
    fn drop(&mut self) {
        self.abort();
    }
}

/// Terminal event of a background file operation spawned by the container
/// file explorer (listing, file read, host copy). Only one operation runs at
/// a time: the presentation layer drains the subscription after each key.
#[derive(Debug)]
pub enum FileOpEvent {
    ListingCompleted {
        result: Result<FileListingDto, AppError>,
    },
    FileReadCompleted {
        path: String,
        result: Result<FileContentDto, AppError>,
    },
    CopyCompleted {
        result: Result<std::path::PathBuf, AppError>,
    },
}

/// Subscription to a background file operation on a container: the spawned
/// task sends exactly one [`FileOpEvent`] before finishing.
///
/// Mirrors [`crate::application::image::ImagePullSubscription`]: the receiver
/// is polled with [`FileOpSubscription::try_recv`] from the UI event loop,
/// and dropping or calling [`FileOpSubscription::abort`] cancels the task.
#[derive(Debug)]
pub struct FileOpSubscription {
    receiver: mpsc::Receiver<FileOpEvent>,
    task: Option<JoinHandle<()>>,
    pending_completed: Option<FileOpEvent>,
}

impl FileOpSubscription {
    pub fn new(receiver: mpsc::Receiver<FileOpEvent>, task: JoinHandle<()>) -> Self {
        FileOpSubscription {
            receiver,
            task: Some(task),
            pending_completed: None,
        }
    }

    /// Subscription whose result is already known; used when the operation
    /// cannot even start (e.g. unresolvable destination directory) and in
    /// tests.
    pub fn completed(event: FileOpEvent) -> Self {
        let (_sender, receiver) = mpsc::channel(1);
        FileOpSubscription {
            receiver,
            task: None,
            pending_completed: Some(event),
        }
    }

    pub fn try_recv(&mut self) -> Result<FileOpEvent, mpsc::error::TryRecvError> {
        if let Some(event) = self.pending_completed.take() {
            return Ok(event);
        }
        self.receiver.try_recv()
    }

    pub fn abort(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

impl Drop for FileOpSubscription {
    fn drop(&mut self) {
        self.abort();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntryDto {
    pub name: String,
    pub path: String,
    pub kind: FileKind,
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
    pub permissions: String,
    pub link_target: Option<String>,
}

impl FileEntryDto {
    /// Display name: directories and symlinks are suffixed, mirroring `ls`.
    pub fn display_name(&self) -> String {
        match self.kind {
            FileKind::Directory => format!("{}/", self.name),
            FileKind::Symlink => format!(
                "{} -> {}",
                self.name,
                self.link_target.as_deref().unwrap_or("?")
            ),
            _ => self.name.clone(),
        }
    }

    pub fn is_dir(&self) -> bool {
        self.kind == FileKind::Directory
    }

    pub fn size_display(&self) -> String {
        if self.is_dir() {
            return String::new();
        }
        ByteSize::new(self.size as i64).human_readable()
    }

    pub fn modified_display(&self) -> String {
        self.modified
            .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default()
    }

    pub fn permissions_display(&self) -> String {
        if self.permissions.is_empty() {
            String::from("-")
        } else {
            self.permissions.clone()
        }
    }
}

/// A file download fetched from a container: raw bytes plus the real file
/// size (bytes may be truncated to a read limit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileContentDto {
    pub data: Vec<u8>,
    pub total_size: u64,
    pub truncated: bool,
}

impl FileContentDto {
    pub fn text(&self) -> Option<String> {
        if self.data.contains(&0) {
            return None;
        }
        String::from_utf8(self.data.clone()).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileListingDto {
    pub container_id: String,
    pub container_name: String,
    pub path: String,
    pub entries: Vec<FileEntryDto>,
}

impl FileListingDto {
    pub fn parent_path(&self) -> Option<String> {
        parent_path(&self.path)
    }
}

/// Parent path of an absolute container path: `/a/b` -> `/a`, `/a` -> `/`,
/// `/` -> `None`.
pub fn parent_path(path: &str) -> Option<String> {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() || trimmed == "/" {
        return None;
    }
    match trimmed.rfind('/') {
        Some(0) => Some("/".to_string()),
        Some(index) => Some(trimmed[..index].to_string()),
        None => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortMappingSpec {
    pub host_port: u16,
    pub container_port: u16,
}

impl std::fmt::Display for PortMappingSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host_port, self.container_port)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateContainerRequest {
    pub name: String,
    pub image: String,
    pub port_mappings: Vec<PortMappingSpec>,
}

impl CreateContainerRequest {
    pub fn new(name: &str, image: &str, port_mappings: &[PortMappingSpec]) -> Self {
        CreateContainerRequest {
            name: name.to_string(),
            image: image.to_string(),
            port_mappings: port_mappings.to_vec(),
        }
    }
}

pub fn parse_port_mappings(input: &str) -> Result<Vec<PortMappingSpec>, AppError> {
    let mut mappings = Vec::new();
    for entry in input.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let (host, container) = entry
            .split_once(':')
            .ok_or_else(|| invalid_port_mapping(entry, "expected <host>:<internal>"))?;
        let host_port = parse_port(host, entry)?;
        let container_port = parse_port(container, entry)?;
        mappings.push(PortMappingSpec {
            host_port,
            container_port,
        });
    }
    Ok(mappings)
}

fn parse_port(value: &str, entry: &str) -> Result<u16, AppError> {
    let port = value
        .parse::<u16>()
        .map_err(|_| invalid_port_mapping(entry, "ports must be between 1 and 65535"))?;
    if port == 0 {
        return Err(invalid_port_mapping(
            entry,
            "ports must be between 1 and 65535",
        ));
    }
    Ok(port)
}

fn invalid_port_mapping(entry: &str, reason: &str) -> AppError {
    AppError::operation_failed(format!("invalid port mapping \"{entry}\": {reason}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::container::{FileEntry, FileKind, FileListing, FilePermissions};
    use chrono::TimeZone;

    fn make_runtime_stats() -> ContainerRuntimeStatsDto {
        ContainerRuntimeStatsDto {
            cpu_percent: 12.34,
            memory_usage: ByteSize::new(512 * 1024 * 1024),
            memory_limit: ByteSize::new(1024 * 1024 * 1024),
            memory_percent: 50.0,
            network_rx: ByteSize::new(2048),
            network_tx: ByteSize::new(1024),
        }
    }

    #[test]
    fn container_runtime_stats_display_methods_format_values() {
        let stats = make_runtime_stats();

        assert_eq!(stats.cpu_display(), "12.3%");
        assert_eq!(stats.memory_list_display(), "512.00 MB (50%)");
        assert_eq!(
            stats.memory_details_display(),
            "512.00 MB / 1.00 GB (50.0%)"
        );
        assert_eq!(stats.network_io_display(), "RX 2.00 KB / TX 1.00 KB");
    }

    #[test]
    fn memory_list_display_omits_percentage_without_limit() {
        let mut stats = make_runtime_stats();
        stats.memory_limit = ByteSize::new(0);
        stats.memory_percent = 0.0;

        assert_eq!(stats.memory_list_display(), "512.00 MB");
        assert_eq!(stats.memory_details_display(), "512.00 MB");
    }

    #[test]
    fn parse_port_mappings_parses_semicolon_separated_entries() {
        let mappings = parse_port_mappings("8080:80; 443:443").unwrap();
        assert_eq!(
            mappings,
            vec![
                PortMappingSpec {
                    host_port: 8080,
                    container_port: 80
                },
                PortMappingSpec {
                    host_port: 443,
                    container_port: 443
                }
            ]
        );
    }

    #[test]
    fn parse_port_mappings_ignores_empty_entries() {
        assert!(parse_port_mappings("").unwrap().is_empty());
        assert!(parse_port_mappings(";; ;").unwrap().is_empty());
    }

    #[test]
    fn parse_port_mappings_rejects_missing_colon() {
        let error = parse_port_mappings("8080").unwrap_err();
        assert!(error.to_string().contains("invalid port mapping \"8080\""));
    }

    #[test]
    fn parse_port_mappings_rejects_non_numeric_port() {
        assert!(parse_port_mappings("abc:80").is_err());
        assert!(parse_port_mappings("8080:xyz").is_err());
    }

    #[test]
    fn parse_port_mappings_rejects_out_of_range_port() {
        assert!(parse_port_mappings("99999:80").is_err());
        assert!(parse_port_mappings("65536:80").is_err());
        assert!(parse_port_mappings("0:80").is_err());
        assert!(parse_port_mappings("8080:0").is_err());
    }

    #[test]
    fn port_mapping_spec_display() {
        let spec = PortMappingSpec {
            host_port: 8080,
            container_port: 80,
        };
        assert_eq!(spec.to_string(), "8080:80");
    }

    #[test]
    fn log_line_display_line_prefixes_timestamp_when_present() {
        let line = LogLineDto {
            timestamp: Some(Utc.with_ymd_and_hms(2026, 10, 26, 10, 30, 0).unwrap()),
            message: "hello".to_string(),
        };
        assert_eq!(line.display_line(), "2026-10-26 10:30:00 hello");
    }

    #[test]
    fn log_line_display_line_is_message_only_without_timestamp() {
        let line = LogLineDto {
            timestamp: None,
            message: "hello".to_string(),
        };
        assert_eq!(line.display_line(), "hello");
    }

    fn make_entry(kind: FileKind, name: &str) -> FileEntryDto {
        FileEntryDto {
            name: name.to_string(),
            path: format!("/dir/{name}"),
            kind,
            size: 1536,
            modified: Some(Utc.with_ymd_and_hms(2026, 10, 26, 10, 30, 0).unwrap()),
            permissions: "-rw-r--r--".to_string(),
            link_target: None,
        }
    }

    #[test]
    fn file_entry_display_name_suffixes_directories_and_symlinks() {
        let dir = make_entry(FileKind::Directory, "etc");
        assert_eq!(dir.display_name(), "etc/");

        let mut link = make_entry(FileKind::Symlink, "lib64");
        link.link_target = Some("/usr/lib64".to_string());
        assert_eq!(link.display_name(), "lib64 -> /usr/lib64");

        let file = make_entry(FileKind::File, "app.conf");
        assert_eq!(file.display_name(), "app.conf");
    }

    #[test]
    fn file_entry_size_display_is_empty_for_directories() {
        assert_eq!(make_entry(FileKind::Directory, "etc").size_display(), "");
        assert_eq!(
            make_entry(FileKind::File, "app.conf").size_display(),
            "1.50 KB"
        );
    }

    #[test]
    fn file_entry_modified_and_permissions_display_fallbacks() {
        let mut entry = make_entry(FileKind::File, "app.conf");
        assert_eq!(entry.modified_display(), "2026-10-26 10:30:00");
        entry.modified = None;
        assert_eq!(entry.modified_display(), "");
        entry.permissions = String::new();
        assert_eq!(entry.permissions_display(), "-");
    }

    #[test]
    fn parent_path_walks_up_to_root() {
        assert_eq!(parent_path("/a/b/c"), Some("/a/b".to_string()));
        assert_eq!(parent_path("/a"), Some("/".to_string()));
        assert_eq!(parent_path("/"), None);
        assert_eq!(parent_path("/a/"), Some("/".to_string()));
    }

    #[test]
    fn file_content_text_rejects_binary_and_invalid_utf8() {
        let text = FileContentDto {
            data: b"hello".to_vec(),
            total_size: 5,
            truncated: false,
        };
        assert_eq!(text.text(), Some("hello".to_string()));

        let binary = FileContentDto {
            data: vec![0, 1, 2],
            total_size: 3,
            truncated: false,
        };
        assert_eq!(binary.text(), None);

        let invalid = FileContentDto {
            data: vec![0xff, 0xfe],
            total_size: 2,
            truncated: false,
        };
        assert_eq!(invalid.text(), None);
    }

    #[test]
    fn file_op_subscription_completed_delivers_event_without_task() {
        let listing = FileListingDto {
            container_id: "abc123".to_string(),
            container_name: "web".to_string(),
            path: "/".to_string(),
            entries: vec![FileEntryDto {
                name: "etc".to_string(),
                path: "/etc".to_string(),
                kind: FileKind::Directory,
                size: 0,
                modified: None,
                permissions: "drwxr-xr-x".to_string(),
                link_target: None,
            }],
        };
        let mut subscription = FileOpSubscription::completed(FileOpEvent::ListingCompleted {
            result: Ok(listing),
        });
        let event = subscription.try_recv().expect("completed event delivered");
        match event {
            FileOpEvent::ListingCompleted { result } => {
                assert_eq!(result.unwrap().path, "/");
            }
            other => panic!("expected ListingCompleted, got {other:?}"),
        }
        // The pending event is consumed exactly once.
        assert!(subscription.try_recv().is_err());
    }

    #[tokio::test]
    async fn file_op_subscription_abort_cancels_task() {
        let (sender, receiver) = mpsc::channel(1);
        let task = tokio::spawn(async move {
            let send = sender.send(FileOpEvent::CopyCompleted {
                result: Ok(std::path::PathBuf::from("/tmp/x")),
            });
            send.await.ok();
            std::future::pending::<()>().await;
        });
        let mut subscription = FileOpSubscription::new(receiver, task);
        subscription.abort();
        // A second abort is a no-op.
        subscription.abort();
    }

    #[test]
    fn file_op_event_debug_and_file_listing_dto_fields() {
        let listing = FileListing::new(
            "/",
            vec![FileEntry::new(
                "a",
                "/a",
                FileKind::File,
                1,
                None,
                FilePermissions::new(0o100644),
            )
            .unwrap()],
        )
        .unwrap();
        assert_eq!(listing.path(), "/");
        let event = FileOpEvent::ListingCompleted {
            result: Ok(listing_dto(&listing)),
        };
        let debug = format!("{event:?}");
        assert!(debug.contains("ListingCompleted"));
    }

    fn listing_dto(listing: &FileListing) -> FileListingDto {
        FileListingDto {
            container_id: "abc123".to_string(),
            container_name: "web".to_string(),
            path: listing.path().to_string(),
            entries: vec![],
        }
    }

    #[test]
    fn container_dto_display_methods_fallback_to_na_without_stats() {
        let container = ContainerDto {
            id: "abc123".to_string(),
            short_id: "abc123".to_string(),
            name: "web".to_string(),
            image: "nginx:latest".to_string(),
            state: ContainerState::Running,
            status: "Up".to_string(),
            created: "2024-01-01".to_string(),
            ports: "80/tcp".to_string(),
            networks: "bridge".to_string(),
            can_start: false,
            can_stop: true,
            can_delete: false,
            can_restart: true,
            can_pause: true,
            can_unpause: false,
            env_vars: vec![],
            runtime_stats: None,
        };

        assert_eq!(container.cpu_display(), "N/A");
        assert_eq!(container.memory_display(), "N/A");
        assert_eq!(container.network_io_display(), "N/A");
    }
}
