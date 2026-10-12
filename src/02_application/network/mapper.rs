use crate::application::network::dto::{NetworkContainerDto, NetworkDto};
use crate::domain::network::Network;

pub struct NetworkMapper;

impl NetworkMapper {
    pub fn to_dto(network: &Network) -> NetworkDto {
        NetworkDto {
            id: network.id().to_string(),
            name: network.name().to_string(),
            driver: network.driver().to_string(),
            scope: network.scope().as_str().to_string(),
            internal: network.internal(),
            attachable: network.attachable(),
            subnet: network.subnet().map(str::to_string),
            gateway: network.gateway().map(str::to_string),
            created: network
                .created()
                .map(|c| c.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            builtin: network.builtin(),
            containers: network
                .containers()
                .iter()
                .map(|container| NetworkContainerDto {
                    name: container.name().to_string(),
                    ip: container.ip().to_string(),
                })
                .collect(),
            can_delete: network.can_be_deleted(),
            container_count: network.containers().len(),
        }
    }

    pub fn to_dto_list(networks: &[Network]) -> Vec<NetworkDto> {
        networks.iter().map(Self::to_dto).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::network::{NetworkContainer, NetworkId, NetworkScope};
    use chrono::{TimeZone, Utc};

    fn create_test_network() -> Network {
        Network::new(
            NetworkId::new("net-abc123").unwrap(),
            "my-network",
            "bridge",
            NetworkScope::Local,
        )
        .with_internal(true)
        .with_attachable(false)
        .with_subnet("172.20.0.0/16".to_string())
        .with_gateway("172.20.0.1".to_string())
        .with_created(Utc.with_ymd_and_hms(2024, 6, 1, 9, 0, 0).unwrap())
        .with_containers(vec![NetworkContainer::new("web", "172.20.0.2")])
    }

    #[test]
    fn to_dto_maps_basic_fields() {
        let network = create_test_network();
        let dto = NetworkMapper::to_dto(&network);

        assert_eq!(dto.id, "net-abc123");
        assert_eq!(dto.name, "my-network");
        assert_eq!(dto.driver, "bridge");
        assert_eq!(dto.scope, "local");
        assert!(dto.internal);
        assert!(!dto.attachable);
        assert_eq!(dto.subnet.as_deref(), Some("172.20.0.0/16"));
        assert_eq!(dto.gateway.as_deref(), Some("172.20.0.1"));
        assert_eq!(dto.created, "2024-06-01 09:00:00");
    }

    #[test]
    fn to_dto_maps_containers_and_derived_counts() {
        let network = create_test_network();
        let dto = NetworkMapper::to_dto(&network);

        assert_eq!(dto.container_count, 1);
        assert_eq!(dto.containers.len(), 1);
        assert_eq!(dto.containers[0].name, "web");
        assert_eq!(dto.containers[0].ip, "172.20.0.2");
        assert!(!dto.can_delete);
    }

    #[test]
    fn to_dto_empty_network_can_delete() {
        let network = Network::new(
            NetworkId::new("net-free").unwrap(),
            "free-network",
            "bridge",
            NetworkScope::Local,
        );
        let dto = NetworkMapper::to_dto(&network);

        assert!(dto.can_delete);
        assert_eq!(dto.container_count, 0);
        assert!(dto.containers.is_empty());
        assert_eq!(dto.created, "N/A");
        assert_eq!(dto.subnet, None);
        assert_eq!(dto.gateway, None);
    }

    #[test]
    fn to_dto_builtin_network_cannot_delete() {
        let network = Network::new(
            NetworkId::new("bridge").unwrap(),
            "bridge",
            "bridge",
            NetworkScope::Local,
        )
        .with_builtin(true);
        let dto = NetworkMapper::to_dto(&network);

        assert!(dto.builtin);
        assert!(!dto.can_delete);
    }

    #[test]
    fn to_dto_list_maps_multiple_networks() {
        let networks = vec![
            create_test_network(),
            Network::new(
                NetworkId::new("net-second").unwrap(),
                "overlay-net",
                "overlay",
                NetworkScope::Swarm,
            ),
        ];
        let dtos = NetworkMapper::to_dto_list(&networks);

        assert_eq!(dtos.len(), 2);
        assert_eq!(dtos[0].name, "my-network");
        assert_eq!(dtos[1].name, "overlay-net");
        assert_eq!(dtos[1].scope, "swarm");
    }

    #[test]
    fn to_dto_list_empty_input() {
        let dtos = NetworkMapper::to_dto_list(&[]);
        assert!(dtos.is_empty());
    }
}
