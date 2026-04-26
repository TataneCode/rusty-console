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
}

#[derive(Debug, Clone)]
pub struct StackDto {
    pub name: String,
    pub container_count: usize,
    pub running_count: usize,
    pub containers: Vec<StackContainerDto>,
}
