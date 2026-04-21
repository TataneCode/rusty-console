use super::value_objects::StackName;
use crate::container::domain::Container;

#[derive(Debug, Clone)]
pub struct Stack {
    name: StackName,
    containers: Vec<Container>,
}

impl Stack {
    pub fn new(name: StackName, containers: Vec<Container>) -> Self {
        Stack { name, containers }
    }

    pub fn name(&self) -> &StackName {
        &self.name
    }

    pub fn containers(&self) -> &[Container] {
        &self.containers
    }

    pub fn container_count(&self) -> usize {
        self.containers.len()
    }

    pub fn running_count(&self) -> usize {
        self.containers.iter().filter(|c| c.is_running()).count()
    }

    pub fn container_ids(&self) -> Vec<String> {
        self.containers.iter().map(|c| c.id().to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::domain::{ContainerId, ContainerState};
    use chrono::Utc;

    fn make_container(state: ContainerState) -> Container {
        Container::new(
            ContainerId::new("abc123").unwrap(),
            "test",
            "nginx:latest",
            state,
            "Up",
            Utc::now(),
        )
    }

    #[test]
    fn test_container_count() {
        let stack = Stack::new(
            StackName::new("my-app").unwrap(),
            vec![
                make_container(ContainerState::Running),
                make_container(ContainerState::Stopped),
            ],
        );
        assert_eq!(stack.container_count(), 2);
    }

    #[test]
    fn test_running_count() {
        let stack = Stack::new(
            StackName::new("my-app").unwrap(),
            vec![
                make_container(ContainerState::Running),
                make_container(ContainerState::Running),
                make_container(ContainerState::Stopped),
            ],
        );
        assert_eq!(stack.running_count(), 2);
    }

    #[test]
    fn test_empty_stack() {
        let stack = Stack::new(StackName::new("empty").unwrap(), vec![]);
        assert_eq!(stack.container_count(), 0);
        assert_eq!(stack.running_count(), 0);
        assert!(stack.container_ids().is_empty());
    }
}
