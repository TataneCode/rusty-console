use crate::application::container::dto::ContainerDto;
use crate::domain::Container;

pub struct ContainerMapper;

impl ContainerMapper {
    pub fn to_dto(container: &Container) -> ContainerDto {
        ContainerDto {
            id: container.id().to_string(),
            name: container.display_name().to_string(),
            image: container.image().to_string(),
            state: container.state(),
            status: container.status().to_string(),
            created: container.created().format("%Y-%m-%d %H:%M:%S").to_string(),
            ports: container.ports_display(),
            networks: container
                .networks()
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            can_start: container.can_be_started(),
            can_stop: container.can_be_stopped(),
            can_delete: container.can_be_deleted(),
            can_restart: container.can_be_restarted(),
            can_pause: container.can_be_paused(),
            can_unpause: container.can_be_unpaused(),
        }
    }

    pub fn to_dto_list(containers: &[Container]) -> Vec<ContainerDto> {
        containers.iter().map(Self::to_dto).collect()
    }
}
