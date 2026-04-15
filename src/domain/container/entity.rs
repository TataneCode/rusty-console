use super::state::ContainerState;
use super::value_objects::{ContainerId, MountInfo, NetworkInfo, PortMapping};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Container {
    id: ContainerId,
    name: String,
    image: String,
    state: ContainerState,
    status: String,
    created: DateTime<Utc>,
    ports: Vec<PortMapping>,
    networks: Vec<NetworkInfo>,
    mounts: Vec<MountInfo>,
    env_vars: Vec<String>,
}

impl Container {
    pub fn new(
        id: ContainerId,
        name: impl Into<String>,
        image: impl Into<String>,
        state: ContainerState,
        status: impl Into<String>,
        created: DateTime<Utc>,
    ) -> Self {
        Container {
            id,
            name: name.into(),
            image: image.into(),
            state,
            status: status.into(),
            created,
            ports: Vec::new(),
            networks: Vec::new(),
            mounts: Vec::new(),
            env_vars: Vec::new(),
        }
    }

    pub fn with_ports(mut self, ports: Vec<PortMapping>) -> Self {
        self.ports = ports;
        self
    }

    pub fn with_networks(mut self, networks: Vec<NetworkInfo>) -> Self {
        self.networks = networks;
        self
    }

    pub fn with_mounts(mut self, mounts: Vec<MountInfo>) -> Self {
        self.mounts = mounts;
        self
    }

    pub fn with_env_vars(mut self, env_vars: Vec<String>) -> Self {
        self.env_vars = env_vars;
        self
    }

    // Getters
    pub fn id(&self) -> &ContainerId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn image(&self) -> &str {
        &self.image
    }

    pub fn state(&self) -> ContainerState {
        self.state
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn created(&self) -> DateTime<Utc> {
        self.created
    }

    pub fn ports(&self) -> &[PortMapping] {
        &self.ports
    }

    pub fn networks(&self) -> &[NetworkInfo] {
        &self.networks
    }

    pub fn mounts(&self) -> &[MountInfo] {
        &self.mounts
    }

    pub fn env_vars(&self) -> &[String] {
        &self.env_vars
    }

    // Business logic
    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }

    pub fn can_be_started(&self) -> bool {
        self.state.can_be_started()
    }

    pub fn can_be_stopped(&self) -> bool {
        self.state.can_be_stopped()
    }

    pub fn can_be_deleted(&self) -> bool {
        self.state.can_be_deleted()
    }

    pub fn can_be_restarted(&self) -> bool {
        self.state.can_be_restarted()
    }

    pub fn can_be_paused(&self) -> bool {
        self.state.can_be_paused()
    }

    pub fn can_be_unpaused(&self) -> bool {
        self.state.can_be_unpaused()
    }

    pub fn display_name(&self) -> &str {
        self.name.trim_start_matches('/')
    }

    pub fn ports_display(&self) -> String {
        if self.ports.is_empty() {
            "-".to_string()
        } else {
            self.ports
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    pub fn uses_volume(&self, volume_name: &str) -> bool {
        self.mounts.iter().any(|m| m.source == volume_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_container(state: ContainerState) -> Container {
        Container::new(
            ContainerId::new("test123").unwrap(),
            "/test-container",
            "nginx:latest",
            state,
            "Up 5 minutes",
            Utc::now(),
        )
    }

    #[test]
    fn test_display_name() {
        let container = create_test_container(ContainerState::Running);
        assert_eq!(container.display_name(), "test-container");
    }

    #[test]
    fn test_can_be_started_when_stopped() {
        let container = create_test_container(ContainerState::Stopped);
        assert!(container.can_be_started());
        assert!(!container.can_be_stopped());
    }

    #[test]
    fn test_can_be_stopped_when_running() {
        let container = create_test_container(ContainerState::Running);
        assert!(!container.can_be_started());
        assert!(container.can_be_stopped());
    }

    #[test]
    fn test_uses_volume() {
        let container = create_test_container(ContainerState::Running)
            .with_mounts(vec![MountInfo::new("my-volume", "/data", "rw")]);
        assert!(container.uses_volume("my-volume"));
        assert!(!container.uses_volume("other-volume"));
    }
}
