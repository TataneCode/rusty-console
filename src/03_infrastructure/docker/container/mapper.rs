use crate::application::container::{ContainerRuntimeStatsDto, ContainerStatsUpdate};
use crate::domain::container::{
    Container, ContainerId, ContainerState, MountInfo, NetworkInfo, PortMapping,
};
use crate::shared::ByteSize;
use bollard::models::ContainerStatsResponse;
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
            .map(|s| {
                s.to_string()
                    .parse::<ContainerState>()
                    .unwrap_or(ContainerState::Stopped)
            })
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

    fn map_ports(ports: Option<&Vec<bollard::models::PortSummary>>) -> Vec<PortMapping> {
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
                        let private_port = parts
                            .first()
                            .and_then(|p| p.parse::<u16>().ok())
                            .unwrap_or(0);
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
                                        PortMapping::new(
                                            private_port,
                                            public_port,
                                            protocol.clone(),
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_else(|| {
                                vec![PortMapping::new(private_port, None, protocol.clone())]
                            })
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

    pub fn stats_update(
        container_id: &str,
        stats: &ContainerStatsResponse,
    ) -> ContainerStatsUpdate {
        let memory_usage = Self::memory_working_set(stats);
        let (network_rx, network_tx) = Self::network_totals(stats);
        ContainerStatsUpdate {
            container_id: container_id.to_string(),
            stats: ContainerRuntimeStatsDto {
                cpu_percent: Self::cpu_percent(stats),
                memory_usage: Self::to_byte_size(memory_usage),
                memory_limit: Self::to_byte_size(
                    stats
                        .memory_stats
                        .as_ref()
                        .and_then(|m| m.limit)
                        .unwrap_or(0),
                ),
                memory_percent: Self::memory_percent(stats),
                network_rx: Self::to_byte_size(network_rx),
                network_tx: Self::to_byte_size(network_tx),
            },
        }
    }

    fn cpu_percent(stats: &ContainerStatsResponse) -> f64 {
        let cpu_total = stats
            .cpu_stats
            .as_ref()
            .and_then(|c| c.cpu_usage.as_ref())
            .and_then(|u| u.total_usage)
            .unwrap_or(0);
        let precpu_total = stats
            .precpu_stats
            .as_ref()
            .and_then(|c| c.cpu_usage.as_ref())
            .and_then(|u| u.total_usage)
            .unwrap_or(0);
        let cpu_delta = cpu_total.saturating_sub(precpu_total);

        let system_delta = stats
            .cpu_stats
            .as_ref()
            .and_then(|c| c.system_cpu_usage)
            .unwrap_or(0)
            .saturating_sub(
                stats
                    .precpu_stats
                    .as_ref()
                    .and_then(|c| c.system_cpu_usage)
                    .unwrap_or(0),
            );

        let cpu_count = stats
            .cpu_stats
            .as_ref()
            .and_then(|c| c.online_cpus)
            .map(|n| n as u64)
            .unwrap_or_else(|| {
                stats
                    .cpu_stats
                    .as_ref()
                    .and_then(|c| c.cpu_usage.as_ref())
                    .and_then(|u| u.percpu_usage.as_ref())
                    .map(|usages| usages.len() as u64)
                    .unwrap_or(1)
            });

        if cpu_delta == 0 || system_delta == 0 {
            0.0
        } else {
            (cpu_delta as f64 / system_delta as f64) * cpu_count as f64 * 100.0
        }
    }

    fn memory_working_set(stats: &ContainerStatsResponse) -> u64 {
        let usage = stats
            .memory_stats
            .as_ref()
            .and_then(|m| m.usage)
            .unwrap_or(0);
        let page_cache = stats
            .memory_stats
            .as_ref()
            .and_then(|m| m.stats.as_ref())
            .and_then(|s| {
                // cgroups v1 uses "total_inactive_file"; cgroups v2 uses "inactive_file"
                s.get("total_inactive_file")
                    .or_else(|| s.get("inactive_file"))
                    .copied()
            })
            .unwrap_or(0);
        usage.saturating_sub(page_cache)
    }

    fn memory_percent(stats: &ContainerStatsResponse) -> f64 {
        let usage = Self::memory_working_set(stats);
        let limit = stats
            .memory_stats
            .as_ref()
            .and_then(|m| m.limit)
            .unwrap_or(0);

        if usage == 0 || limit == 0 {
            0.0
        } else {
            usage as f64 / limit as f64 * 100.0
        }
    }

    fn network_totals(stats: &ContainerStatsResponse) -> (u64, u64) {
        stats
            .networks
            .as_ref()
            .map(|networks| {
                networks.values().fold((0u64, 0u64), |(rx, tx), network| {
                    (
                        rx.saturating_add(network.rx_bytes.unwrap_or(0)),
                        tx.saturating_add(network.tx_bytes.unwrap_or(0)),
                    )
                })
            })
            .unwrap_or((0, 0))
    }

    fn to_byte_size(bytes: u64) -> ByteSize {
        ByteSize::new(bytes.min(i64::MAX as u64) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::{
        ContainerBlkioStats, ContainerConfig, ContainerCpuStats, ContainerCpuUsage,
        ContainerInspectResponse, ContainerMemoryStats, ContainerNetworkStats, ContainerPidsStats,
        ContainerState as BollardContainerState, ContainerStateStatusEnum, ContainerStatsResponse,
        ContainerStorageStats, ContainerSummary, ContainerSummaryNetworkSettings,
        ContainerSummaryStateEnum, ContainerThrottlingData, EndpointSettings, MountPoint,
        NetworkSettings, PortBinding, PortSummary, PortSummaryTypeEnum,
    };
    use std::collections::HashMap;

    fn make_summary(
        id: &str,
        name: &str,
        image: &str,
        state: ContainerSummaryStateEnum,
    ) -> ContainerSummary {
        ContainerSummary {
            id: Some(id.to_string()),
            names: Some(vec![format!("/{name}")]),
            image: Some(image.to_string()),
            state: Some(state),
            status: Some("Up 5 minutes".to_string()),
            created: Some(1_700_000_000),
            ..Default::default()
        }
    }

    fn make_empty_blkio_stats() -> ContainerBlkioStats {
        ContainerBlkioStats {
            ..Default::default()
        }
    }

    fn make_empty_throttling_data() -> ContainerThrottlingData {
        ContainerThrottlingData {
            periods: Some(0),
            throttled_periods: Some(0),
            throttled_time: Some(0),
        }
    }

    // ── from_docker tests ──

    #[test]
    fn from_docker_happy_path() {
        let summary = ContainerSummary {
            id: Some("abc123def456".to_string()),
            names: Some(vec!["/my-container".to_string()]),
            image: Some("nginx:latest".to_string()),
            state: Some(ContainerSummaryStateEnum::RUNNING),
            status: Some("Up 5 minutes".to_string()),
            created: Some(1_700_000_000),
            ports: Some(vec![PortSummary {
                private_port: 80,
                public_port: Some(8080),
                typ: Some(PortSummaryTypeEnum::TCP),
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
                PortSummary {
                    private_port: 80,
                    public_port: Some(8080),
                    typ: Some(PortSummaryTypeEnum::TCP),
                    ..Default::default()
                },
                PortSummary {
                    private_port: 443,
                    public_port: None,
                    typ: Some(PortSummaryTypeEnum::TCP),
                    ..Default::default()
                },
                PortSummary {
                    private_port: 53,
                    public_port: Some(5353),
                    typ: Some(PortSummaryTypeEnum::UDP),
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

        let mut summary =
            make_summary("abc123", "web", "nginx", ContainerSummaryStateEnum::RUNNING);
        summary.network_settings = Some(ContainerSummaryNetworkSettings {
            networks: Some(networks),
        });

        let container = ContainerInfraMapper::from_docker(&summary).unwrap();
        assert_eq!(container.networks().len(), 2);
        let net_names: Vec<&str> = container
            .networks()
            .iter()
            .map(|n| n.name.as_str())
            .collect();
        assert!(net_names.contains(&"bridge"));
        assert!(net_names.contains(&"custom_net"));
    }

    #[test]
    fn from_docker_mount_extraction() {
        let mut summary = make_summary(
            "abc123",
            "db",
            "postgres",
            ContainerSummaryStateEnum::RUNNING,
        );
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
        assert_eq!(
            container.mounts()[0].destination,
            "/var/lib/postgresql/data"
        );
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
        assert_eq!(
            container.env_vars()[0],
            "DATABASE_URL=postgres://localhost/db"
        );
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

    #[test]
    fn stats_update_calculates_cpu_memory_and_networks() {
        let stats = ContainerStatsResponse {
            id: Some("abc123".to_string()),
            name: Some("web".to_string()),
            pids_stats: Some(ContainerPidsStats {
                current: Some(1),
                limit: Some(10),
            }),
            networks: Some(HashMap::from([
                (
                    "eth0".to_string(),
                    ContainerNetworkStats {
                        rx_bytes: Some(2_048),
                        tx_bytes: Some(1_024),
                        ..Default::default()
                    },
                ),
                (
                    "eth1".to_string(),
                    ContainerNetworkStats {
                        rx_bytes: Some(1_024),
                        tx_bytes: Some(512),
                        ..Default::default()
                    },
                ),
            ])),
            memory_stats: Some(ContainerMemoryStats {
                stats: None,
                usage: Some(512 * 1024 * 1024),
                limit: Some(1024 * 1024 * 1024),
                ..Default::default()
            }),
            blkio_stats: Some(make_empty_blkio_stats()),
            cpu_stats: Some(ContainerCpuStats {
                cpu_usage: Some(ContainerCpuUsage {
                    percpu_usage: Some(vec![0, 0]),
                    total_usage: Some(300),
                    ..Default::default()
                }),
                system_cpu_usage: Some(1_000),
                online_cpus: Some(2),
                throttling_data: Some(make_empty_throttling_data()),
            }),
            precpu_stats: Some(ContainerCpuStats {
                cpu_usage: Some(ContainerCpuUsage {
                    percpu_usage: Some(vec![0, 0]),
                    total_usage: Some(100),
                    ..Default::default()
                }),
                system_cpu_usage: Some(500),
                online_cpus: Some(2),
                throttling_data: Some(make_empty_throttling_data()),
            }),
            storage_stats: Some(ContainerStorageStats {
                ..Default::default()
            }),
            ..Default::default()
        };

        let update = ContainerInfraMapper::stats_update("abc123", &stats);

        assert_eq!(update.container_id, "abc123");
        assert_eq!(update.stats.cpu_display(), "80.0%");
        assert_eq!(update.stats.memory_list_display(), "512.00 MB (50%)");
        assert_eq!(update.stats.network_io_display(), "RX 3.00 KB / TX 1.50 KB");
    }

    #[test]
    fn stats_update_returns_zero_network_when_no_networks() {
        let stats = ContainerStatsResponse {
            id: Some("abc123".to_string()),
            name: Some("web".to_string()),
            networks: None,
            memory_stats: Some(ContainerMemoryStats {
                usage: Some(256),
                limit: None,
                ..Default::default()
            }),
            blkio_stats: Some(make_empty_blkio_stats()),
            cpu_stats: Some(ContainerCpuStats {
                cpu_usage: Some(ContainerCpuUsage {
                    percpu_usage: None,
                    total_usage: Some(100),
                    ..Default::default()
                }),
                system_cpu_usage: Some(100),
                online_cpus: None,
                throttling_data: Some(make_empty_throttling_data()),
            }),
            precpu_stats: Some(ContainerCpuStats {
                cpu_usage: Some(ContainerCpuUsage {
                    percpu_usage: None,
                    total_usage: Some(100),
                    ..Default::default()
                }),
                system_cpu_usage: Some(100),
                online_cpus: None,
                throttling_data: Some(make_empty_throttling_data()),
            }),
            ..Default::default()
        };

        let update = ContainerInfraMapper::stats_update("abc123", &stats);

        assert_eq!(update.stats.cpu_display(), "0.0%");
        assert_eq!(update.stats.memory_details_display(), "256 B");
        assert_eq!(update.stats.network_io_display(), "RX 0 B / TX 0 B");
    }

    fn make_base_stats(
        usage: u64,
        limit: u64,
        mem_stats: Option<HashMap<String, u64>>,
    ) -> ContainerStatsResponse {
        ContainerStatsResponse {
            id: Some("abc123".to_string()),
            name: Some("web".to_string()),
            pids_stats: Some(ContainerPidsStats {
                current: Some(1),
                limit: Some(100),
            }),
            networks: None,
            memory_stats: Some(ContainerMemoryStats {
                stats: mem_stats,
                usage: Some(usage),
                limit: Some(limit),
                ..Default::default()
            }),
            blkio_stats: Some(make_empty_blkio_stats()),
            cpu_stats: Some(ContainerCpuStats {
                cpu_usage: Some(ContainerCpuUsage {
                    percpu_usage: None,
                    total_usage: Some(100),
                    ..Default::default()
                }),
                system_cpu_usage: Some(1_000),
                online_cpus: Some(1),
                throttling_data: Some(make_empty_throttling_data()),
            }),
            precpu_stats: Some(ContainerCpuStats {
                cpu_usage: Some(ContainerCpuUsage {
                    percpu_usage: None,
                    total_usage: Some(0),
                    ..Default::default()
                }),
                system_cpu_usage: Some(0),
                online_cpus: Some(1),
                throttling_data: Some(make_empty_throttling_data()),
            }),
            storage_stats: Some(ContainerStorageStats {
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn stats_update_subtracts_total_inactive_file_for_cgroups_v1() {
        let mem_stats = HashMap::from([
            ("total_inactive_file".to_string(), 128 * 1024 * 1024u64), // 128 MB page cache
        ]);
        let stats = make_base_stats(
            512 * 1024 * 1024,  // 512 MB raw usage
            1024 * 1024 * 1024, // 1 GB limit
            Some(mem_stats),
        );

        let update = ContainerInfraMapper::stats_update("abc123", &stats);

        // Working set = 512MB - 128MB = 384MB
        assert_eq!(update.stats.memory_usage.bytes(), 384 * 1024 * 1024);
        // Percent = 384 / 1024 = 37.5%
        assert!((update.stats.memory_percent - 37.5).abs() < 0.1);
    }

    #[test]
    fn stats_update_subtracts_inactive_file_for_cgroups_v2() {
        let mem_stats = HashMap::from([
            ("inactive_file".to_string(), 64 * 1024 * 1024u64), // 64 MB page cache
        ]);
        let stats = make_base_stats(
            256 * 1024 * 1024,  // 256 MB raw usage
            1024 * 1024 * 1024, // 1 GB limit
            Some(mem_stats),
        );

        let update = ContainerInfraMapper::stats_update("abc123", &stats);

        // Working set = 256MB - 64MB = 192MB
        assert_eq!(update.stats.memory_usage.bytes(), 192 * 1024 * 1024);
        // Percent = 192 / 1024 = 18.75%
        assert!((update.stats.memory_percent - 18.75).abs() < 0.1);
    }

    #[test]
    fn stats_update_uses_raw_usage_when_no_memory_substats() {
        let stats = make_base_stats(
            300 * 1024 * 1024, // 300 MB raw usage, no sub-stats
            1024 * 1024 * 1024,
            None,
        );

        let update = ContainerInfraMapper::stats_update("abc123", &stats);

        assert_eq!(update.stats.memory_usage.bytes(), 300 * 1024 * 1024);
    }
}
