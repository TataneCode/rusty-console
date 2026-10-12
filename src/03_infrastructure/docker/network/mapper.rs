use crate::domain::network::{Network, NetworkContainer, NetworkId, NetworkScope};
use bollard::models::{Network as BollardNetwork, NetworkInspect};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct NetworkInfraMapper;

impl NetworkInfraMapper {
    /// Docker exposes predefined networks (`bridge`, `host`, `none`) either by
    /// name or through the `com.docker.network.bridge.name` option.
    pub fn is_builtin(name: &str, options: Option<&HashMap<String, String>>) -> bool {
        matches!(name, "bridge" | "host" | "none")
            || options.is_some_and(|opts| opts.contains_key("com.docker.network.bridge.name"))
    }

    pub fn from_docker(network: &BollardNetwork) -> Option<Network> {
        let name = network.name.clone()?;
        let id = network.id.clone()?;
        let driver = network.driver.clone()?;
        let id = NetworkId::new(id).ok()?;
        let scope = NetworkScope::from_docker(network.scope.as_deref().unwrap_or("local"));
        let (subnet, gateway) = subnet_and_gateway(network.ipam.as_ref());

        let mut entity = Network::new(id, name.clone(), driver, scope)
            .with_internal(network.internal.unwrap_or(false))
            .with_attachable(network.attachable.unwrap_or(false))
            .with_builtin(Self::is_builtin(&name, network.options.as_ref()));

        if let Some(subnet) = subnet {
            entity = entity.with_subnet(subnet);
        }
        if let Some(gateway) = gateway {
            entity = entity.with_gateway(gateway);
        }
        if let Some(created) = parse_created(network.created.as_deref()) {
            entity = entity.with_created(created);
        }

        Some(entity)
    }

    pub fn from_inspect(inspect: &NetworkInspect) -> Option<Network> {
        let name = inspect.name.clone()?;
        let id = inspect.id.clone()?;
        let driver = inspect.driver.clone()?;
        let id = NetworkId::new(id).ok()?;
        let scope = NetworkScope::from_docker(inspect.scope.as_deref().unwrap_or("local"));
        let (subnet, gateway) = subnet_and_gateway(inspect.ipam.as_ref());

        let mut entity = Network::new(id, name.clone(), driver, scope)
            .with_internal(inspect.internal.unwrap_or(false))
            .with_attachable(inspect.attachable.unwrap_or(false))
            .with_builtin(Self::is_builtin(&name, inspect.options.as_ref()))
            .with_containers(Self::containers(inspect));

        if let Some(subnet) = subnet {
            entity = entity.with_subnet(subnet);
        }
        if let Some(gateway) = gateway {
            entity = entity.with_gateway(gateway);
        }
        if let Some(created) = parse_created(inspect.created.as_deref()) {
            entity = entity.with_created(created);
        }

        Some(entity)
    }

    fn containers(inspect: &NetworkInspect) -> Vec<NetworkContainer> {
        let mut containers: Vec<NetworkContainer> = inspect
            .containers
            .as_ref()
            .map(|endpoints| {
                endpoints
                    .iter()
                    .map(|(container_id, endpoint)| {
                        NetworkContainer::new(
                            endpoint
                                .name
                                .clone()
                                .unwrap_or_else(|| container_id.clone()),
                            endpoint
                                .ipv4_address
                                .clone()
                                .unwrap_or_else(|| "N/A".to_string()),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        containers.sort_unstable_by(|a, b| a.name().cmp(b.name()));
        containers
    }
}

fn subnet_and_gateway(ipam: Option<&bollard::models::Ipam>) -> (Option<String>, Option<String>) {
    ipam.and_then(|ipam| ipam.config.as_ref())
        .and_then(|configs| configs.first())
        .map(|config| (config.subnet.clone(), config.gateway.clone()))
        .unwrap_or((None, None))
}

fn parse_created(created: Option<&str>) -> Option<DateTime<Utc>> {
    created
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::{EndpointResource, Ipam, IpamConfig};

    fn make_bollard_network(name: &str) -> BollardNetwork {
        BollardNetwork {
            name: Some(name.to_string()),
            id: Some(format!("id-{name}")),
            driver: Some("bridge".to_string()),
            scope: Some("local".to_string()),
            created: Some("2024-01-15T10:30:00Z".to_string()),
            internal: Some(false),
            attachable: Some(false),
            ipam: Some(Ipam {
                config: Some(vec![IpamConfig {
                    subnet: Some("172.20.0.0/16".to_string()),
                    gateway: Some("172.20.0.1".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn make_bollard_inspect(name: &str) -> NetworkInspect {
        NetworkInspect {
            name: Some(name.to_string()),
            id: Some(format!("id-{name}")),
            driver: Some("bridge".to_string()),
            scope: Some("local".to_string()),
            created: Some("2024-01-15T10:30:00Z".to_string()),
            containers: Some(HashMap::from([
                (
                    "container-id-1".to_string(),
                    EndpointResource {
                        name: Some("web".to_string()),
                        ipv4_address: Some("172.20.0.2/16".to_string()),
                        ..Default::default()
                    },
                ),
                (
                    "container-id-2".to_string(),
                    EndpointResource {
                        name: Some("api".to_string()),
                        ipv4_address: Some("172.20.0.3/16".to_string()),
                        ..Default::default()
                    },
                ),
            ])),
            ipam: Some(Ipam {
                config: Some(vec![IpamConfig {
                    subnet: Some("172.20.0.0/16".to_string()),
                    gateway: Some("172.20.0.1".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn from_docker_maps_basic_fields() {
        let network = NetworkInfraMapper::from_docker(&make_bollard_network("app-net")).unwrap();

        assert_eq!(network.name(), "app-net");
        assert_eq!(network.driver(), "bridge");
        assert_eq!(network.scope(), NetworkScope::Local);
        assert_eq!(network.subnet(), Some("172.20.0.0/16"));
        assert_eq!(network.gateway(), Some("172.20.0.1"));
        assert!(network.created().is_some());
        assert!(!network.builtin());
        assert!(network.can_be_deleted());
    }

    #[test]
    fn from_docker_skips_network_without_name() {
        let mut network = make_bollard_network("app-net");
        network.name = None;
        assert!(NetworkInfraMapper::from_docker(&network).is_none());
    }

    #[test]
    fn from_inspect_maps_connected_containers_sorted() {
        let network = NetworkInfraMapper::from_inspect(&make_bollard_inspect("app-net")).unwrap();

        assert_eq!(network.containers().len(), 2);
        assert_eq!(network.containers()[0].name(), "api");
        assert_eq!(network.containers()[0].ip(), "172.20.0.3/16");
        assert_eq!(network.containers()[1].name(), "web");
        assert!(!network.can_be_deleted());
        assert!(network.can_disconnect());
    }

    #[test]
    fn builtin_detection_by_name_and_by_option() {
        assert!(NetworkInfraMapper::is_builtin("bridge", None));
        assert!(NetworkInfraMapper::is_builtin("host", None));
        assert!(NetworkInfraMapper::is_builtin("none", None));
        assert!(!NetworkInfraMapper::is_builtin("app-net", None));

        let mut options = HashMap::new();
        options.insert(
            "com.docker.network.bridge.name".to_string(),
            "docker0".to_string(),
        );
        assert!(NetworkInfraMapper::is_builtin("docker0", Some(&options)));
        assert!(!NetworkInfraMapper::is_builtin(
            "docker0",
            Some(&HashMap::new())
        ));
    }

    #[test]
    fn from_docker_marks_builtin_network_undeletable() {
        let network = NetworkInfraMapper::from_docker(&make_bollard_network("bridge")).unwrap();
        assert!(network.builtin());
        assert!(!network.can_be_deleted());
    }

    #[test]
    fn missing_ipam_leaves_subnet_and_gateway_unset() {
        let mut network = make_bollard_network("app-net");
        network.ipam = None;
        let entity = NetworkInfraMapper::from_docker(&network).unwrap();

        assert_eq!(entity.subnet(), None);
        assert_eq!(entity.gateway(), None);
    }
}
