use crate::application::container::{
    ContainerRepository, ContainerStatsEvent, ContainerStatsSubscription, CreateContainerRequest,
};
use crate::application::error::AppError;
use crate::application::shared::PruneResultDto;
use crate::domain::container::Container;
use crate::infrastructure::docker::client::DockerClient;
use crate::infrastructure::docker::container::mapper::ContainerInfraMapper;
use crate::infrastructure::error::InfraError;
use async_trait::async_trait;
use bollard::query_parameters::{
    CreateContainerOptions, ListContainersOptions, LogsOptions, PruneContainersOptions,
    RemoveContainerOptions, RestartContainerOptions, StartContainerOptions, StatsOptions,
    StopContainerOptions,
};
use bollard::models::{ContainerCreateBody, HostConfig, PortBinding};
use futures_util::StreamExt;
use std::collections::HashMap;
use tokio::sync::mpsc;

const STATS_EVENT_BUFFER: usize = 128;

pub struct ContainerAdapter {
    docker: DockerClient,
}

impl ContainerAdapter {
    pub fn new(docker: DockerClient) -> Self {
        ContainerAdapter { docker }
    }

    async fn collect_logs(&self, id: &str, tail: Option<usize>) -> Result<String, AppError> {
        let options = LogsOptions {
            stdout: true,
            stderr: true,
            timestamps: true,
            tail: tail.map(|t| t.to_string()).unwrap_or_default(),
            ..Default::default()
        };
        let mut stream = self.docker.inner().logs(id, Some(options));
        let mut logs = String::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    logs.push_str(&output.to_string());
                }
                Err(e) => {
                    return Err(AppError::from(InfraError::from(e)));
                }
            }
        }
        Ok(logs)
    }
}

#[async_trait]
impl ContainerRepository for ContainerAdapter {
    async fn get_all(&self) -> Result<Vec<Container>, AppError> {
        let mut filters = HashMap::new();
        filters.insert(
            "status".to_string(),
            vec![
                "created".to_string(),
                "restarting".to_string(),
                "running".to_string(),
                "removing".to_string(),
                "paused".to_string(),
                "exited".to_string(),
                "dead".to_string(),
            ],
        );

        let options = ListContainersOptions {
            all: true,
            filters: Some(filters),
            ..Default::default()
        };

        let containers = self
            .docker
            .inner()
            .list_containers(Some(options))
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        Ok(containers
            .iter()
            .filter_map(ContainerInfraMapper::from_docker)
            .collect())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Container>, AppError> {
        let response = self
            .docker
            .inner()
            .inspect_container(id, None)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        Ok(ContainerInfraMapper::from_inspect(&response))
    }

    async fn export_logs(&self, id: &str) -> Result<String, AppError> {
        let options = LogsOptions {
            stdout: true,
            stderr: true,
            timestamps: true,
            ..Default::default()
        };
        let mut stream = self.docker.inner().logs(id, Some(options));
        let mut logs = String::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(output) => {
                    logs.push_str(&output.to_string());
                }
                Err(e) => {
                    return Err(AppError::from(InfraError::from(e)));
                }
            }
        }
        Ok(logs)
    }

    async fn get_logs(&self, id: &str, tail: Option<usize>) -> Result<String, AppError> {
        self.collect_logs(id, Some(tail.unwrap_or(100))).await
    }

    async fn create(&self, request: &CreateContainerRequest) -> Result<String, AppError> {
        let options = CreateContainerOptions {
            name: Some(request.name.clone()),
            platform: String::new(),
        };
        let mut port_bindings: bollard::models::PortMap = HashMap::new();
        for mapping in &request.port_mappings {
            port_bindings.insert(
                format!("{}/tcp", mapping.container_port),
                Some(vec![PortBinding {
                    host_ip: None,
                    host_port: Some(mapping.host_port.to_string()),
                }]),
            );
        }
        let host_config = if port_bindings.is_empty() {
            HostConfig::default()
        } else {
            HostConfig {
                port_bindings: Some(port_bindings),
                ..Default::default()
            }
        };
        let config = ContainerCreateBody {
            image: Some(request.image.clone()),
            host_config: Some(host_config),
            ..Default::default()
        };
        let response = self
            .docker
            .inner()
            .create_container(Some(options), config)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;
        Ok(response.id)
    }

    async fn start(&self, id: &str) -> Result<(), AppError> {
        self.docker
            .inner()
            .start_container(id, None::<StartContainerOptions>)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)
    }

    async fn stop(&self, id: &str) -> Result<(), AppError> {
        let options = StopContainerOptions {
            t: Some(10),
            ..Default::default()
        };

        self.docker
            .inner()
            .stop_container(id, Some(options))
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)
    }

    async fn delete(&self, id: &str, force: bool) -> Result<(), AppError> {
        let options = RemoveContainerOptions {
            force,
            ..Default::default()
        };

        self.docker
            .inner()
            .remove_container(id, Some(options))
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)
    }

    async fn restart(&self, id: &str) -> Result<(), AppError> {
        let options = RestartContainerOptions {
            t: Some(10),
            ..Default::default()
        };

        self.docker
            .inner()
            .restart_container(id, Some(options))
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)
    }

    async fn pause(&self, id: &str) -> Result<(), AppError> {
        self.docker
            .inner()
            .pause_container(id)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)
    }

    async fn unpause(&self, id: &str) -> Result<(), AppError> {
        self.docker
            .inner()
            .unpause_container(id)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)
    }

    async fn prune(&self) -> Result<PruneResultDto, AppError> {
        let result = self
            .docker
            .inner()
            .prune_containers(None::<PruneContainersOptions>)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        Ok(PruneResultDto {
            deleted_count: result
                .containers_deleted
                .map(|c| c.len() as u32)
                .unwrap_or(0),
            space_freed: result.space_reclaimed.unwrap_or(0) as u64,
        })
    }

    async fn subscribe_stats(
        &self,
        container_ids: Vec<String>,
    ) -> Result<ContainerStatsSubscription, AppError> {
        let (sender, receiver) = mpsc::channel(STATS_EVENT_BUFFER);
        let tasks = container_ids
            .into_iter()
            .map(|container_id| {
                let docker = self.docker.clone();
                let sender = sender.clone();

                tokio::spawn(async move {
                    let mut stream = docker.inner().stats(
                        &container_id,
                        Some(StatsOptions {
                            stream: true,
                            one_shot: false,
                        }),
                    );

                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(stats) => {
                                let update =
                                    ContainerInfraMapper::stats_update(&container_id, &stats);
                                if sender
                                    .send(ContainerStatsEvent::Update(update))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            Err(error) => {
                                let message = AppError::from(InfraError::from(error)).to_string();
                                let _ = sender
                                    .send(ContainerStatsEvent::Error {
                                        container_id: container_id.clone(),
                                        message,
                                    })
                                    .await;
                                break;
                            }
                        }
                    }
                })
            })
            .collect();
        drop(sender);

        Ok(ContainerStatsSubscription::new(receiver, tasks))
    }
}
