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
