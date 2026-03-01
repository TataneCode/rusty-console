use crate::domain::{Container, ContainerId, ContainerState, MountInfo, NetworkInfo, PortMapping};
use bollard::models::ContainerSummary;
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
}
