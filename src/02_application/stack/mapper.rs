use crate::application::stack::dto::{StackContainerDto, StackDto};
use crate::domain::stack::{Stack, StackContainer};

pub struct StackMapper;

impl StackMapper {
    fn to_container_dto(container: &StackContainer) -> StackContainerDto {
        StackContainerDto {
            id: container.id().to_string(),
            name: container.display_name().to_string(),
            image: container.image().to_string(),
            state: container.state(),
            status: container.status().to_string(),
            ports: container.ports().to_string(),
            can_start: container.can_be_started(),
            can_stop: container.can_be_stopped(),
            can_restart: container.state().can_be_restarted(),
            can_pause: container.state().can_be_paused(),
            can_unpause: container.state().can_be_unpaused(),
            runtime_stats: None,
        }
    }

    pub fn to_dto(stack: &Stack) -> StackDto {
        StackDto {
            name: stack.name().to_string(),
            container_count: stack.container_count(),
            running_count: stack.running_count(),
            containers: stack
                .containers()
                .iter()
                .map(Self::to_container_dto)
                .collect(),
        }
    }

    pub fn to_dto_list(stacks: &[Stack]) -> Vec<StackDto> {
        stacks.iter().map(Self::to_dto).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::stack::{StackContainer, StackContainerState, StackName};

    fn make_container(state: StackContainerState) -> StackContainer {
        StackContainer::new("abc123", "/web", "nginx:latest", state, "Up", "80/tcp")
    }

    #[test]
    fn test_to_dto_counts() {
        let stack = Stack::new(
            StackName::new("my-app").unwrap(),
            vec![
                make_container(StackContainerState::Running),
                make_container(StackContainerState::Stopped),
            ],
        );
        let dto = StackMapper::to_dto(&stack);
        assert_eq!(dto.name, "my-app");
        assert_eq!(dto.container_count, 2);
        assert_eq!(dto.running_count, 1);
        assert_eq!(dto.containers.len(), 2);
        assert_eq!(dto.containers[0].name, "web");
        assert!(!dto.containers[0].can_start);
        assert!(dto.containers[0].can_stop);
        assert!(dto.containers[0].can_restart);
        assert!(dto.containers[0].can_pause);
        assert!(!dto.containers[0].can_unpause);
        assert!(dto.containers[0].runtime_stats.is_none());
    }

    #[test]
    fn test_to_dto_capabilities_follow_state() {
        let stopped = make_container(StackContainerState::Stopped);
        let paused = make_container(StackContainerState::Paused);
        let stack = Stack::new(StackName::new("my-app").unwrap(), vec![stopped, paused]);
        let dto = StackMapper::to_dto(&stack);

        let stopped = &dto.containers[0];
        assert!(stopped.can_start);
        assert!(!stopped.can_stop);
        assert!(!stopped.can_restart);
        assert!(!stopped.can_pause);
        assert!(!stopped.can_unpause);

        let paused = &dto.containers[1];
        assert!(!paused.can_start);
        assert!(paused.can_stop);
        assert!(paused.can_restart);
        assert!(!paused.can_pause);
        assert!(paused.can_unpause);
    }

    #[test]
    fn test_to_dto_list() {
        let stacks = vec![
            Stack::new(StackName::new("app-a").unwrap(), vec![]),
            Stack::new(StackName::new("app-b").unwrap(), vec![]),
        ];
        let dtos = StackMapper::to_dto_list(&stacks);
        assert_eq!(dtos.len(), 2);
        assert_eq!(dtos[0].name, "app-a");
        assert_eq!(dtos[1].name, "app-b");
    }
}
