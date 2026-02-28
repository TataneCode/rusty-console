use crate::domain::ContainerState;

#[derive(Debug, Clone)]
pub struct ContainerDto {
    pub id: String,
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
}

#[derive(Debug, Clone)]
pub struct ContainerLogsDto {
    pub container_id: String,
    pub container_name: String,
    pub logs: String,
}
