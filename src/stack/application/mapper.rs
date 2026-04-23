use crate::container::application::mapper::ContainerMapper;
use crate::stack::application::dto::StackDto;
use crate::stack::domain::Stack;

pub struct StackMapper;

impl StackMapper {
    pub fn to_dto(stack: &Stack) -> StackDto {
        StackDto {
            name: stack.name().to_string(),
            container_count: stack.container_count(),
            running_count: stack.running_count(),
            containers: ContainerMapper::to_dto_list(stack.containers()),
        }
    }

    pub fn to_dto_list(stacks: &[Stack]) -> Vec<StackDto> {
        stacks.iter().map(Self::to_dto).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::domain::{Container, ContainerId, ContainerState};
    use crate::stack::domain::StackName;
    use chrono::Utc;

    fn make_container(state: ContainerState) -> Container {
        Container::new(
            ContainerId::new("abc123").unwrap(),
            "web",
            "nginx:latest",
            state,
            "Up",
            Utc::now(),
        )
    }

    #[test]
    fn test_to_dto_counts() {
        let stack = Stack::new(
            StackName::new("my-app").unwrap(),
            vec![
                make_container(ContainerState::Running),
                make_container(ContainerState::Stopped),
            ],
        );
        let dto = StackMapper::to_dto(&stack);
        assert_eq!(dto.name, "my-app");
        assert_eq!(dto.container_count, 2);
        assert_eq!(dto.running_count, 1);
        assert_eq!(dto.containers.len(), 2);
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
