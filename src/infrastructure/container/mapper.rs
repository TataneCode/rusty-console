use crate::domain::{Container, ContainerId, ContainerState, MountInfo, NetworkInfo, PortMapping};
use bollard::models::{ContainerInspectResponse, ContainerSummary};
use chrono::{TimeZone, Utc};

pub struct ContainerInfraMapper;

impl ContainerInfraMapper {
    pub fn from_docker(summary: &ContainerSummary) -> Option<Container> {
        let id = summary.id.as_ref()?;
        let container_id = ContainerId::new(id).ok()?;

        let name = summary
            .names
            .as_ref()
            .and_then(|n| n.first())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let image = summary
            .image
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        let state = summary
            .state
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(ContainerState::Stopped);

        let status = summary.status.clone().unwrap_or_default();

        let created = summary
            .created
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
            .unwrap_or_else(Utc::now);

        let ports = Self::map_ports(summary.ports.as_ref());
        let networks = Self::map_networks(summary);
        let mounts = Self::map_mounts(summary);

        Some(
            Container::new(container_id, name, image, state, status, created)
                .with_ports(ports)
                .with_networks(networks)
                .with_mounts(mounts),
        )
    }

    fn map_ports(ports: Option<&Vec<bollard::models::Port>>) -> Vec<PortMapping> {
        ports
            .map(|ports| {
                ports
                    .iter()
                    .map(|p| {
                        let protocol = p
                            .typ
                            .as_ref()
                            .map(|t| format!("{:?}", t).to_lowercase())
                            .unwrap_or_else(|| "tcp".to_string());
                        PortMapping::new(p.private_port, p.public_port, protocol)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn map_networks(summary: &ContainerSummary) -> Vec<NetworkInfo> {
        summary
            .network_settings
            .as_ref()
            .and_then(|ns| ns.networks.as_ref())
            .map(|networks| {
                networks
                    .iter()
                    .map(|(name, endpoint)| {
                        NetworkInfo::new(
                            name.clone(),
                            endpoint.ip_address.clone().unwrap_or_default(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn map_mounts(summary: &ContainerSummary) -> Vec<MountInfo> {
        summary
            .mounts
            .as_ref()
            .map(|mounts| {
                mounts
                    .iter()
                    .filter_map(|m| {
                        Some(MountInfo::new(
                            m.name.clone().or_else(|| m.source.clone())?,
                            m.destination.clone()?,
                            m.mode.clone().unwrap_or_else(|| "rw".to_string()),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn from_inspect(response: &ContainerInspectResponse) -> Option<Container> {
        let id = response.id.as_ref()?;
        let container_id = ContainerId::new(id).ok()?;

        let name = response
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        let image = response
            .config
            .as_ref()
            .and_then(|c| c.image.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let state_info = response.state.as_ref();

        let state = state_info
            .and_then(|s| s.status.as_ref())
            .map(|s| format!("{:?}", s).to_lowercase())
            .and_then(|s| s.parse().ok())
            .unwrap_or(ContainerState::Stopped);

        let status = state_info
            .and_then(|s| s.status.as_ref())
            .map(|s| format!("{:?}", s))
            .unwrap_or_default();

        let created = response
            .created
            .as_ref()
            .and_then(|c| chrono::DateTime::parse_from_rfc3339(c).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let env_vars = response
            .config
            .as_ref()
            .and_then(|c| c.env.clone())
            .unwrap_or_default();

        let ports = Self::map_inspect_ports(response);
        let networks = Self::map_inspect_networks(response);
        let mounts = Self::map_inspect_mounts(response);

        Some(
            Container::new(container_id, name, image, state, status, created)
                .with_ports(ports)
                .with_networks(networks)
                .with_mounts(mounts)
                .with_env_vars(env_vars),
        )
    }

    fn map_inspect_ports(response: &ContainerInspectResponse) -> Vec<PortMapping> {
        response
            .network_settings
            .as_ref()
            .and_then(|ns| ns.ports.as_ref())
            .map(|ports| {
                ports
                    .iter()
                    .flat_map(|(port_key, bindings)| {
                        let parts: Vec<&str> = port_key.split('/').collect();
                        let private_port = parts.first().and_then(|p| p.parse::<u16>().ok()).unwrap_or(0);
                        let protocol = parts.get(1).unwrap_or(&"tcp").to_string();

                        bindings
                            .as_ref()
                            .map(|bs| {
                                bs.iter()
                                    .map(|b| {
                                        let public_port = b
                                            .host_port
                                            .as_ref()
                                            .and_then(|p| p.parse::<u16>().ok());
                                        PortMapping::new(private_port, public_port, protocol.clone())
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_else(|| vec![PortMapping::new(private_port, None, protocol.clone())])
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn map_inspect_networks(response: &ContainerInspectResponse) -> Vec<NetworkInfo> {
        response
            .network_settings
            .as_ref()
            .and_then(|ns| ns.networks.as_ref())
            .map(|networks| {
                networks
                    .iter()
                    .map(|(name, endpoint)| {
                        NetworkInfo::new(
                            name.clone(),
                            endpoint.ip_address.clone().unwrap_or_default(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn map_inspect_mounts(response: &ContainerInspectResponse) -> Vec<MountInfo> {
        response
            .mounts
            .as_ref()
            .map(|mounts| {
                mounts
                    .iter()
                    .filter_map(|m| {
                        Some(MountInfo::new(
                            m.name.clone().or_else(|| m.source.clone())?,
                            m.destination.clone()?,
                            m.mode.clone().unwrap_or_else(|| "rw".to_string()),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::{
        ContainerConfig, ContainerInspectResponse, ContainerState as BollardContainerState,
        ContainerStateStatusEnum, ContainerSummary, ContainerSummaryNetworkSettings,
        EndpointSettings, MountPoint, NetworkSettings, Port, PortBinding, PortTypeEnum,
    };
    use std::collections::HashMap;

    fn make_summary(id: &str, name: &str, image: &str, state: &str) -> ContainerSummary {
        ContainerSummary {
            id: Some(id.to_string()),
            names: Some(vec![format!("/{name}")]),
            image: Some(image.to_string()),
            state: Some(state.to_string()),
            status: Some("Up 5 minutes".to_string()),
            created: Some(1_700_000_000),
            ..Default::default()
        }
    }

    // ── from_docker tests ──

    #[test]
    fn from_docker_happy_path() {
        let summary = ContainerSummary {
            id: Some("abc123def456".to_string()),
            names: Some(vec!["/my-container".to_string()]),
            image: Some("nginx:latest".to_string()),
            state: Some("running".to_string()),
            status: Some("Up 5 minutes".to_string()),
            created: Some(1_700_000_000),
            ports: Some(vec![Port {
                private_port: 80,
                public_port: Some(8080),
                typ: Some(PortTypeEnum::TCP),
                ..Default::default()
            }]),
            ..Default::default()
        };

        let container = ContainerInfraMapper::from_docker(&summary).unwrap();
        assert_eq!(container.id().as_str(), "abc123def456");
        assert_eq!(container.name(), "/my-container");
        assert_eq!(container.image(), "nginx:latest");
        assert_eq!(container.state(), ContainerState::Running);
        assert_eq!(container.status(), "Up 5 minutes");
        assert_eq!(container.ports().len(), 1);
        assert_eq!(container.ports()[0].private_port, 80);
        assert_eq!(container.ports()[0].public_port, Some(8080));
    }

    #[test]
    fn from_docker_missing_id_returns_none() {
        let summary = ContainerSummary {
            id: None,
            names: Some(vec!["/c".to_string()]),
            ..Default::default()
        };
        assert!(ContainerInfraMapper::from_docker(&summary).is_none());
    }

    #[test]
    fn from_docker_empty_id_returns_none() {
        let summary = ContainerSummary {
            id: Some(String::new()),
            names: Some(vec!["/c".to_string()]),
            ..Default::default()
        };
        assert!(ContainerInfraMapper::from_docker(&summary).is_none());
    }

    #[test]
    fn from_docker_missing_names_defaults_to_unknown() {
        let summary = ContainerSummary {
            id: Some("abc123".to_string()),
            names: None,
            ..Default::default()
        };
        let container = ContainerInfraMapper::from_docker(&summary).unwrap();
        assert_eq!(container.name(), "unknown");
    }

    #[test]
    fn from_docker_port_mapping_with_multiple_ports() {
        let summary = ContainerSummary {
            id: Some("abc123".to_string()),
            names: Some(vec!["/web".to_string()]),
            ports: Some(vec![
                Port {
                    private_port: 80,
                    public_port: Some(8080),
                    typ: Some(PortTypeEnum::TCP),
                    ..Default::default()
                },
                Port {
                    private_port: 443,
                    public_port: None,
                    typ: Some(PortTypeEnum::TCP),
                    ..Default::default()
                },
                Port {
                    private_port: 53,
                    public_port: Some(5353),
                    typ: Some(PortTypeEnum::UDP),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        let container = ContainerInfraMapper::from_docker(&summary).unwrap();
        assert_eq!(container.ports().len(), 3);
        assert_eq!(container.ports()[0].public_port, Some(8080));
        assert_eq!(container.ports()[1].public_port, None);
        assert_eq!(container.ports()[2].protocol, "udp");
    }

    #[test]
    fn from_docker_network_extraction() {
        let mut networks = HashMap::new();
        networks.insert(
            "bridge".to_string(),
            EndpointSettings {
                ip_address: Some("172.17.0.2".to_string()),
                ..Default::default()
            },
        );
        networks.insert(
            "custom_net".to_string(),
            EndpointSettings {
                ip_address: Some("10.0.0.5".to_string()),
                ..Default::default()
            },
        );

        let mut summary = make_summary("abc123", "web", "nginx", "running");
        summary.network_settings = Some(ContainerSummaryNetworkSettings {
            networks: Some(networks),
        });

        let container = ContainerInfraMapper::from_docker(&summary).unwrap();
        assert_eq!(container.networks().len(), 2);
        let net_names: Vec<&str> = container.networks().iter().map(|n| n.name.as_str()).collect();
        assert!(net_names.contains(&"bridge"));
        assert!(net_names.contains(&"custom_net"));
    }

    #[test]
    fn from_docker_mount_extraction() {
        let mut summary = make_summary("abc123", "db", "postgres", "running");
        summary.mounts = Some(vec![
            MountPoint {
                name: Some("pgdata".to_string()),
                destination: Some("/var/lib/postgresql/data".to_string()),
                mode: Some("rw".to_string()),
                ..Default::default()
            },
            MountPoint {
                name: None,
                source: Some("/host/config".to_string()),
                destination: Some("/etc/app".to_string()),
                mode: None,
                ..Default::default()
            },
        ]);

        let container = ContainerInfraMapper::from_docker(&summary).unwrap();
        assert_eq!(container.mounts().len(), 2);
        assert_eq!(container.mounts()[0].source, "pgdata");
        assert_eq!(container.mounts()[0].destination, "/var/lib/postgresql/data");
        assert_eq!(container.mounts()[1].source, "/host/config");
        assert_eq!(container.mounts()[1].mode, "rw"); // default when None
    }

    // ── from_inspect tests ──

    #[test]
    fn from_inspect_happy_path_with_env_vars() {
        let response = ContainerInspectResponse {
            id: Some("abc123def456".to_string()),
            name: Some("/my-app".to_string()),
            created: Some("2024-01-15T10:30:00Z".to_string()),
            state: Some(BollardContainerState {
                status: Some(ContainerStateStatusEnum::RUNNING),
                ..Default::default()
            }),
            config: Some(ContainerConfig {
                image: Some("myapp:v2".to_string()),
                env: Some(vec![
                    "DATABASE_URL=postgres://localhost/db".to_string(),
                    "RUST_LOG=info".to_string(),
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let container = ContainerInfraMapper::from_inspect(&response).unwrap();
        assert_eq!(container.id().as_str(), "abc123def456");
        assert_eq!(container.name(), "/my-app");
        assert_eq!(container.image(), "myapp:v2");
        assert_eq!(container.state(), ContainerState::Running);
        assert_eq!(container.env_vars().len(), 2);
        assert_eq!(container.env_vars()[0], "DATABASE_URL=postgres://localhost/db");
        assert_eq!(container.env_vars()[1], "RUST_LOG=info");
    }

    #[test]
    fn from_inspect_missing_optional_fields() {
        let response = ContainerInspectResponse {
            id: Some("def456".to_string()),
            name: None,
            created: None,
            state: None,
            config: None,
            ..Default::default()
        };

        let container = ContainerInfraMapper::from_inspect(&response).unwrap();
        assert_eq!(container.name(), "unknown");
        assert_eq!(container.image(), "unknown");
        assert_eq!(container.state(), ContainerState::Stopped);
        assert!(container.env_vars().is_empty());
    }

    #[test]
    fn from_inspect_missing_id_returns_none() {
        let response = ContainerInspectResponse {
            id: None,
            ..Default::default()
        };
        assert!(ContainerInfraMapper::from_inspect(&response).is_none());
    }

    #[test]
    fn from_inspect_ports_from_network_settings() {
        let mut ports = HashMap::new();
        ports.insert(
            "80/tcp".to_string(),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some("8080".to_string()),
            }]),
        );
        ports.insert("443/tcp".to_string(), None);

        let response = ContainerInspectResponse {
            id: Some("abc123".to_string()),
            network_settings: Some(NetworkSettings {
                ports: Some(ports),
                ..Default::default()
            }),
            ..Default::default()
        };

        let container = ContainerInfraMapper::from_inspect(&response).unwrap();
        assert_eq!(container.ports().len(), 2);
        let has_public_80 = container
            .ports()
            .iter()
            .any(|p| p.private_port == 80 && p.public_port == Some(8080));
        let has_no_public_443 = container
            .ports()
            .iter()
            .any(|p| p.private_port == 443 && p.public_port.is_none());
        assert!(has_public_80);
        assert!(has_no_public_443);
    }
}
