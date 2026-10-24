use crate::application::error::AppError;
use crate::domain::container::ContainerState;
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
        use chrono::TimeZone;
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
