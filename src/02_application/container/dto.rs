use crate::domain::container::ContainerState;
use crate::shared::ByteSize;
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
    pub logs: String,
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
