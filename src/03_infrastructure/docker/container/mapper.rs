use crate::application::container::{ContainerRuntimeStatsDto, ContainerStatsUpdate};
use crate::domain::container::{
    Container, ContainerId, ContainerState, MountInfo, NetworkInfo, PortMapping,
};
use crate::shared::ByteSize;
use bollard::container::{MemoryStatsStats, Stats};
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

    pub fn stats_update(container_id: &str, stats: &Stats) -> ContainerStatsUpdate {
        let memory_usage = Self::memory_working_set(stats);
        let (network_rx, network_tx) = Self::network_totals(stats);
        ContainerStatsUpdate {
            container_id: container_id.to_string(),
            stats: ContainerRuntimeStatsDto {
                cpu_percent: Self::cpu_percent(stats),
                memory_usage: Self::to_byte_size(memory_usage),
                memory_limit: Self::to_byte_size(stats.memory_stats.limit.unwrap_or(0)),
                memory_percent: Self::memory_percent(stats),
                network_rx: Self::to_byte_size(network_rx),
                network_tx: Self::to_byte_size(network_tx),
            },
        }
    }

    fn cpu_percent(stats: &Stats) -> f64 {
        let cpu_delta = stats
            .cpu_stats
            .cpu_usage
            .total_usage
            .saturating_sub(stats.precpu_stats.cpu_usage.total_usage);
        let system_delta = stats
            .cpu_stats
            .system_cpu_usage
            .unwrap_or(0)
            .saturating_sub(stats.precpu_stats.system_cpu_usage.unwrap_or(0));
        let cpu_count = stats.cpu_stats.online_cpus.unwrap_or_else(|| {
            stats
                .cpu_stats
                .cpu_usage
                .percpu_usage
                .as_ref()
                .map(|usages| usages.len() as u64)
                .unwrap_or(1)
        });

        if cpu_delta == 0 || system_delta == 0 {
            0.0
        } else {
            (cpu_delta as f64 / system_delta as f64) * cpu_count as f64 * 100.0
        }
    }

    fn memory_working_set(stats: &Stats) -> u64 {
        let usage = stats.memory_stats.usage.unwrap_or(0);
        let page_cache = match stats.memory_stats.stats {
            Some(MemoryStatsStats::V1(v1)) => v1.total_inactive_file,
            Some(MemoryStatsStats::V2(v2)) => v2.inactive_file,
            None => 0,
        };
        usage.saturating_sub(page_cache)
    }

    fn memory_percent(stats: &Stats) -> f64 {
        let usage = Self::memory_working_set(stats);
        let limit = stats.memory_stats.limit.unwrap_or(0);

        if usage == 0 || limit == 0 {
            0.0
        } else {
            usage as f64 / limit as f64 * 100.0
        }
    }

    fn network_totals(stats: &Stats) -> (u64, u64) {
        if let Some(networks) = &stats.networks {
            networks.values().fold((0, 0), |(rx, tx), network| {
                (
                    rx.saturating_add(network.rx_bytes),
                    tx.saturating_add(network.tx_bytes),
                )
            })
        } else if let Some(network) = stats.network {
            (network.rx_bytes, network.tx_bytes)
        } else {
            (0, 0)
        }
    }

    fn to_byte_size(bytes: u64) -> ByteSize {
        ByteSize::new(bytes.min(i64::MAX as u64) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::container::{
        BlkioStats, CPUStats, CPUUsage, MemoryStats, MemoryStatsStats, MemoryStatsStatsV1,
        MemoryStatsStatsV2, NetworkStats, PidsStats, Stats, StorageStats, ThrottlingData,
    };
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

    fn make_empty_blkio_stats() -> BlkioStats {
        BlkioStats {
            io_service_bytes_recursive: None,
            io_serviced_recursive: None,
            io_queue_recursive: None,
            io_service_time_recursive: None,
            io_wait_time_recursive: None,
            io_merged_recursive: None,
            io_time_recursive: None,
            sectors_recursive: None,
        }
    }

    fn make_empty_throttling_data() -> ThrottlingData {
        ThrottlingData {
            periods: 0,
            throttled_periods: 0,
            throttled_time: 0,
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
        let stats = Stats {
            id: "abc123".to_string(),
            name: "web".to_string(),
            read: "2024-01-01T00:00:00Z".to_string(),
            preread: "2024-01-01T00:00:00Z".to_string(),
            num_procs: 1,
            pids_stats: PidsStats {
                current: Some(1),
                limit: Some(10),
            },
            network: None,
            networks: Some(HashMap::from([
                (
                    "eth0".to_string(),
                    NetworkStats {
                        rx_dropped: 0,
                        rx_bytes: 2_048,
                        rx_errors: 0,
                        tx_packets: 0,
                        tx_dropped: 0,
                        rx_packets: 0,
                        tx_errors: 0,
                        tx_bytes: 1_024,
                    },
                ),
                (
                    "eth1".to_string(),
                    NetworkStats {
                        rx_dropped: 0,
                        rx_bytes: 1_024,
                        rx_errors: 0,
                        tx_packets: 0,
                        tx_dropped: 0,
                        rx_packets: 0,
                        tx_errors: 0,
                        tx_bytes: 512,
                    },
                ),
            ])),
            memory_stats: MemoryStats {
                stats: None,
                max_usage: None,
                usage: Some(512 * 1024 * 1024),
                failcnt: None,
                limit: Some(1024 * 1024 * 1024),
                commit: None,
                commit_peak: None,
                commitbytes: None,
                commitpeakbytes: None,
                privateworkingset: None,
            },
            blkio_stats: make_empty_blkio_stats(),
            cpu_stats: CPUStats {
                cpu_usage: CPUUsage {
                    percpu_usage: Some(vec![0, 0]),
                    usage_in_usermode: 0,
                    total_usage: 300,
                    usage_in_kernelmode: 0,
                },
                system_cpu_usage: Some(1_000),
                online_cpus: Some(2),
                throttling_data: make_empty_throttling_data(),
            },
            precpu_stats: CPUStats {
                cpu_usage: CPUUsage {
                    percpu_usage: Some(vec![0, 0]),
                    usage_in_usermode: 0,
                    total_usage: 100,
                    usage_in_kernelmode: 0,
                },
                system_cpu_usage: Some(500),
                online_cpus: Some(2),
                throttling_data: make_empty_throttling_data(),
            },
            storage_stats: StorageStats {
                read_count_normalized: None,
                read_size_bytes: None,
                write_count_normalized: None,
                write_size_bytes: None,
            },
        };

        let update = ContainerInfraMapper::stats_update("abc123", &stats);

        assert_eq!(update.container_id, "abc123");
        assert_eq!(update.stats.cpu_display(), "80.0%");
        assert_eq!(update.stats.memory_list_display(), "512.00 MB (50%)");
        assert_eq!(update.stats.network_io_display(), "RX 3.00 KB / TX 1.50 KB");
    }

    #[test]
    fn stats_update_falls_back_to_single_network_and_zero_cpu() {
        let stats = Stats {
            id: "abc123".to_string(),
            name: "web".to_string(),
            read: "2024-01-01T00:00:00Z".to_string(),
            preread: "2024-01-01T00:00:00Z".to_string(),
            num_procs: 1,
            pids_stats: PidsStats {
                current: Some(1),
                limit: Some(10),
            },
            network: Some(NetworkStats {
                rx_dropped: 0,
                rx_bytes: 4_096,
                rx_errors: 0,
                tx_packets: 0,
                tx_dropped: 0,
                rx_packets: 0,
                tx_errors: 0,
                tx_bytes: 2_048,
            }),
            networks: None,
            memory_stats: MemoryStats {
                stats: None,
                max_usage: None,
                usage: Some(256),
                failcnt: None,
                limit: None,
                commit: None,
                commit_peak: None,
                commitbytes: None,
                commitpeakbytes: None,
                privateworkingset: None,
            },
            blkio_stats: make_empty_blkio_stats(),
            cpu_stats: CPUStats {
                cpu_usage: CPUUsage {
                    percpu_usage: None,
                    usage_in_usermode: 0,
                    total_usage: 100,
                    usage_in_kernelmode: 0,
                },
                system_cpu_usage: Some(100),
                online_cpus: None,
                throttling_data: make_empty_throttling_data(),
            },
            precpu_stats: CPUStats {
                cpu_usage: CPUUsage {
                    percpu_usage: None,
                    usage_in_usermode: 0,
                    total_usage: 100,
                    usage_in_kernelmode: 0,
                },
                system_cpu_usage: Some(100),
                online_cpus: None,
                throttling_data: make_empty_throttling_data(),
            },
            storage_stats: StorageStats {
                read_count_normalized: None,
                read_size_bytes: None,
                write_count_normalized: None,
                write_size_bytes: None,
            },
        };

        let update = ContainerInfraMapper::stats_update("abc123", &stats);

        assert_eq!(update.stats.cpu_display(), "0.0%");
        assert_eq!(update.stats.memory_details_display(), "256 B");
        assert_eq!(update.stats.network_io_display(), "RX 4.00 KB / TX 2.00 KB");
    }

    fn make_base_stats(usage: u64, limit: u64, mem_stats: Option<MemoryStatsStats>) -> Stats {
        Stats {
            id: "abc123".to_string(),
            name: "web".to_string(),
            read: "2024-01-01T00:00:00Z".to_string(),
            preread: "2024-01-01T00:00:00Z".to_string(),
            num_procs: 1,
            pids_stats: PidsStats {
                current: Some(1),
                limit: Some(100),
            },
            network: None,
            networks: None,
            memory_stats: MemoryStats {
                stats: mem_stats,
                max_usage: None,
                usage: Some(usage),
                failcnt: None,
                limit: Some(limit),
                commit: None,
                commit_peak: None,
                commitbytes: None,
                commitpeakbytes: None,
                privateworkingset: None,
            },
            blkio_stats: make_empty_blkio_stats(),
            cpu_stats: CPUStats {
                cpu_usage: CPUUsage {
                    percpu_usage: None,
                    usage_in_usermode: 0,
                    total_usage: 100,
                    usage_in_kernelmode: 0,
                },
                system_cpu_usage: Some(1_000),
                online_cpus: Some(1),
                throttling_data: make_empty_throttling_data(),
            },
            precpu_stats: CPUStats {
                cpu_usage: CPUUsage {
                    percpu_usage: None,
                    usage_in_usermode: 0,
                    total_usage: 0,
                    usage_in_kernelmode: 0,
                },
                system_cpu_usage: Some(0),
                online_cpus: Some(1),
                throttling_data: make_empty_throttling_data(),
            },
            storage_stats: StorageStats {
                read_count_normalized: None,
                read_size_bytes: None,
                write_count_normalized: None,
                write_size_bytes: None,
            },
        }
    }

    #[test]
    fn stats_update_subtracts_total_inactive_file_for_cgroups_v1() {
        let v1 = MemoryStatsStatsV1 {
            cache: 0,
            dirty: 0,
            mapped_file: 0,
            total_inactive_file: 128 * 1024 * 1024, // 128 MB page cache
            pgpgout: 0,
            rss: 0,
            total_mapped_file: 0,
            writeback: 0,
            unevictable: 0,
            pgpgin: 0,
            total_unevictable: 0,
            pgmajfault: 0,
            total_rss: 0,
            total_rss_huge: 0,
            total_writeback: 0,
            total_inactive_anon: 0,
            rss_huge: 0,
            hierarchical_memory_limit: 0,
            total_pgfault: 0,
            total_active_file: 0,
            active_anon: 0,
            total_active_anon: 0,
            total_pgpgout: 0,
            total_cache: 0,
            total_dirty: 0,
            inactive_anon: 0,
            active_file: 0,
            pgfault: 0,
            inactive_file: 0,
            total_pgmajfault: 0,
            total_pgpgin: 0,
            hierarchical_memsw_limit: None,
            shmem: None,
            total_shmem: None,
        };
        let stats = make_base_stats(
            512 * 1024 * 1024,  // 512 MB raw usage
            1024 * 1024 * 1024, // 1 GB limit
            Some(MemoryStatsStats::V1(v1)),
        );

        let update = ContainerInfraMapper::stats_update("abc123", &stats);

        // Working set = 512MB - 128MB = 384MB
        assert_eq!(update.stats.memory_usage.bytes(), 384 * 1024 * 1024);
        // Percent = 384 / 1024 = 37.5%
        assert!((update.stats.memory_percent - 37.5).abs() < 0.1);
    }

    #[test]
    fn stats_update_subtracts_inactive_file_for_cgroups_v2() {
        let v2 = MemoryStatsStatsV2 {
            anon: 0,
            file: 0,
            kernel_stack: 0,
            slab: 0,
            sock: 0,
            shmem: 0,
            file_mapped: 0,
            file_dirty: 0,
            file_writeback: 0,
            anon_thp: 0,
            inactive_anon: 0,
            active_anon: 0,
            inactive_file: 64 * 1024 * 1024, // 64 MB page cache
            active_file: 0,
            unevictable: 0,
            slab_reclaimable: 0,
            slab_unreclaimable: 0,
            pgfault: 0,
            pgmajfault: 0,
            workingset_refault: 0,
            workingset_activate: 0,
            workingset_nodereclaim: 0,
            pgrefill: 0,
            pgscan: 0,
            pgsteal: 0,
            pgactivate: 0,
            pgdeactivate: 0,
            pglazyfree: 0,
            pglazyfreed: 0,
            thp_fault_alloc: 0,
            thp_collapse_alloc: 0,
        };
        let stats = make_base_stats(
            256 * 1024 * 1024,  // 256 MB raw usage
            1024 * 1024 * 1024, // 1 GB limit
            Some(MemoryStatsStats::V2(v2)),
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
