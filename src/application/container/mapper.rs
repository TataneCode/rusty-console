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
            env_vars: container.env_vars().to_vec(),
        }
    }

    pub fn to_dto_list(containers: &[Container]) -> Vec<ContainerDto> {
        containers.iter().map(Self::to_dto).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ContainerId, ContainerState, NetworkInfo, PortMapping};
    use chrono::{TimeZone, Utc};

    fn create_running_container() -> Container {
        Container::new(
            ContainerId::new("abc123def456").unwrap(),
            "/my-nginx",
            "nginx:latest",
            ContainerState::Running,
            "Up 5 minutes",
            Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
        )
    }

    fn create_stopped_container() -> Container {
        Container::new(
            ContainerId::new("def456abc789").unwrap(),
            "/my-redis",
            "redis:7",
            ContainerState::Stopped,
            "Exited (0) 2 hours ago",
            Utc.with_ymd_and_hms(2024, 1, 14, 8, 0, 0).unwrap(),
        )
    }

    #[test]
    fn to_dto_maps_basic_fields() {
        let container = create_running_container();
        let dto = ContainerMapper::to_dto(&container);

        assert_eq!(dto.id, "abc123def456");
        assert_eq!(dto.name, "my-nginx");
        assert_eq!(dto.image, "nginx:latest");
        assert_eq!(dto.status, "Up 5 minutes");
        assert_eq!(dto.created, "2024-01-15 10:30:00");
    }

    #[test]
    fn to_dto_strips_slash_prefix_from_name() {
        let container = create_running_container();
        let dto = ContainerMapper::to_dto(&container);
        assert_eq!(dto.name, "my-nginx");
    }

    #[test]
    fn to_dto_maps_ports_and_networks() {
        let container = create_running_container()
            .with_ports(vec![
                PortMapping::new(80, Some(8080), "tcp"),
                PortMapping::new(443, None, "tcp"),
            ])
            .with_networks(vec![
                NetworkInfo::new("bridge", "172.17.0.2"),
                NetworkInfo::new("custom-net", ""),
            ]);

        let dto = ContainerMapper::to_dto(&container);

        assert_eq!(dto.ports, "8080:80/tcp, 443/tcp");
        assert_eq!(dto.networks, "bridge (172.17.0.2), custom-net");
    }

    #[test]
    fn to_dto_empty_ports_shows_dash() {
        let container = create_running_container();
        let dto = ContainerMapper::to_dto(&container);
        assert_eq!(dto.ports, "-");
    }

    #[test]
    fn to_dto_maps_env_vars() {
        let container = create_running_container()
            .with_env_vars(vec!["FOO=bar".to_string(), "DB_HOST=localhost".to_string()]);

        let dto = ContainerMapper::to_dto(&container);

        assert_eq!(dto.env_vars, vec!["FOO=bar", "DB_HOST=localhost"]);
    }

    #[test]
    fn to_dto_running_container_boolean_flags() {
        let container = create_running_container();
        let dto = ContainerMapper::to_dto(&container);

        assert!(!dto.can_start);
        assert!(dto.can_stop);
        assert!(dto.can_pause);
        assert!(!dto.can_unpause);
        assert!(dto.can_restart);
    }

    #[test]
    fn to_dto_stopped_container_boolean_flags() {
        let container = create_stopped_container();
        let dto = ContainerMapper::to_dto(&container);

        assert!(dto.can_start);
        assert!(!dto.can_stop);
        assert!(!dto.can_pause);
        assert!(!dto.can_unpause);
        assert!(!dto.can_restart);
    }

    #[test]
    fn to_dto_paused_container_boolean_flags() {
        let container = Container::new(
            ContainerId::new("paused123").unwrap(),
            "/paused-app",
            "app:1.0",
            ContainerState::Paused,
            "Up 10 minutes (Paused)",
            Utc::now(),
        );
        let dto = ContainerMapper::to_dto(&container);

        assert!(!dto.can_start);
        assert!(dto.can_stop);
        assert!(!dto.can_pause);
        assert!(dto.can_unpause);
        assert!(dto.can_restart);
    }

    #[test]
    fn to_dto_list_maps_multiple_containers() {
        let containers = vec![create_running_container(), create_stopped_container()];
        let dtos = ContainerMapper::to_dto_list(&containers);

        assert_eq!(dtos.len(), 2);
        assert_eq!(dtos[0].name, "my-nginx");
        assert_eq!(dtos[1].name, "my-redis");
    }

    #[test]
    fn to_dto_list_empty_input() {
        let dtos = ContainerMapper::to_dto_list(&[]);
        assert!(dtos.is_empty());
    }
}
