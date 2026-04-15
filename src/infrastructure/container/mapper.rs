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
