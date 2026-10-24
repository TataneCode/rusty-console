use crate::application::container::ContainerRuntimeStatsDto;
use crate::domain::stack::StackContainerState;

#[derive(Debug, Clone)]
pub struct StackContainerDto {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: StackContainerState,
    pub status: String,
    pub ports: String,
    pub can_start: bool,
    pub can_stop: bool,
    pub can_restart: bool,
    pub can_pause: bool,
    pub can_unpause: bool,
    pub runtime_stats: Option<ContainerRuntimeStatsDto>,
}

impl StackContainerDto {
    pub fn state_display(&self) -> &'static str {
        match self.state {
            StackContainerState::Running => "Running",
            StackContainerState::Paused => "Paused",
            StackContainerState::Stopped => "Stopped",
            StackContainerState::Exited => "Exited",
            StackContainerState::Dead => "Dead",
            StackContainerState::Created => "Created",
            StackContainerState::Removing => "Removing",
            StackContainerState::Restarting => "Restarting",
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
}

#[derive(Debug, Clone)]
pub struct StackDto {
    pub name: String,
    pub container_count: usize,
    pub running_count: usize,
    pub containers: Vec<StackContainerDto>,
}
