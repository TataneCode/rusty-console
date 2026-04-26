use super::StackContainerState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackContainer {
    id: String,
    name: String,
    image: String,
    state: StackContainerState,
    status: String,
    ports: String,
}

impl StackContainer {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        image: impl Into<String>,
        state: StackContainerState,
        status: impl Into<String>,
        ports: impl Into<String>,
    ) -> Self {
        StackContainer {
            id: id.into(),
            name: name.into(),
            image: image.into(),
            state,
            status: status.into(),
            ports: ports.into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn image(&self) -> &str {
        &self.image
    }

    pub fn state(&self) -> StackContainerState {
        self.state
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn ports(&self) -> &str {
        &self.ports
    }

    pub fn display_name(&self) -> &str {
        self.name.trim_start_matches('/')
    }

    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }

    pub fn can_be_started(&self) -> bool {
        self.state.can_be_started()
    }

    pub fn can_be_stopped(&self) -> bool {
        self.state.can_be_stopped()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_container(state: StackContainerState) -> StackContainer {
        StackContainer::new(
            "abc123",
            "/web",
            "nginx:latest",
            state,
            "Up 5 minutes",
            "80/tcp",
        )
    }

    #[test]
    fn test_display_name_strips_slash_prefix() {
        let container = make_container(StackContainerState::Running);
        assert_eq!(container.display_name(), "web");
    }

    #[test]
    fn test_running_container_capabilities() {
        let container = make_container(StackContainerState::Running);
        assert!(container.is_running());
        assert!(!container.can_be_started());
        assert!(container.can_be_stopped());
    }

    #[test]
    fn test_stopped_container_capabilities() {
        let container = make_container(StackContainerState::Stopped);
        assert!(!container.is_running());
        assert!(container.can_be_started());
        assert!(!container.can_be_stopped());
    }
}
