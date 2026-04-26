use crate::application::container::ContainerRepository;
use crate::application::error::AppError;
use crate::domain::container::Container;
use crate::infrastructure::docker::client::DockerClient;
use crate::infrastructure::docker::container::mapper::ContainerInfraMapper;
use crate::shared::PruneResultDto;
use async_trait::async_trait;
use bollard::container::{
    ListContainersOptions, LogsOptions, PruneContainersOptions, RemoveContainerOptions,
    RestartContainerOptions, StartContainerOptions, StopContainerOptions,
};
use futures_util::StreamExt;
use std::collections::HashMap;

pub struct ContainerAdapter {
    docker: DockerClient,
}

impl ContainerAdapter {
    pub fn new(docker: DockerClient) -> Self {
        ContainerAdapter { docker }
    }
}

#[async_trait]
impl ContainerRepository for ContainerAdapter {
    async fn get_all(&self) -> Result<Vec<Container>, AppError> {
        let mut filters = HashMap::new();
        filters.insert(
            "status",
            vec![
                "created",
                "restarting",
                "running",
                "removing",
                "paused",
                "exited",
                "dead",
            ],
        );

        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };

        let containers = self
            .docker
            .inner()
            .list_containers(Some(options))
            .await
            .map_err(|e| AppError::repository(e.to_string()))?;

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
            .map_err(|e| AppError::repository(e.to_string()))?;

        Ok(ContainerInfraMapper::from_inspect(&response))
    }

    async fn get_logs(&self, id: &str, tail: Option<usize>) -> Result<String, AppError> {
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail
                .map(|t| t.to_string())
                .unwrap_or_else(|| "100".to_string()),
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
                    return Err(AppError::repository(format!("Failed to get logs: {}", e)));
                }
            }
        }

        Ok(logs)
    }

    async fn start(&self, id: &str) -> Result<(), AppError> {
        self.docker
            .inner()
            .start_container(id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| AppError::operation_failed(format!("Failed to start container: {}", e)))
    }

    async fn stop(&self, id: &str) -> Result<(), AppError> {
        let options = StopContainerOptions { t: 10 };

        self.docker
            .inner()
            .stop_container(id, Some(options))
            .await
            .map_err(|e| AppError::operation_failed(format!("Failed to stop container: {}", e)))
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
            .map_err(|e| AppError::operation_failed(format!("Failed to delete container: {}", e)))
    }

    async fn restart(&self, id: &str) -> Result<(), AppError> {
        let options = RestartContainerOptions { t: 10 };

        self.docker
            .inner()
            .restart_container(id, Some(options))
            .await
            .map_err(|e| AppError::operation_failed(format!("Failed to restart container: {}", e)))
    }

    async fn pause(&self, id: &str) -> Result<(), AppError> {
        self.docker
            .inner()
            .pause_container(id)
            .await
            .map_err(|e| AppError::operation_failed(format!("Failed to pause container: {}", e)))
    }

    async fn unpause(&self, id: &str) -> Result<(), AppError> {
        self.docker
            .inner()
            .unpause_container(id)
            .await
            .map_err(|e| AppError::operation_failed(format!("Failed to unpause container: {}", e)))
    }

    async fn prune(&self) -> Result<PruneResultDto, AppError> {
        let result = self
            .docker
            .inner()
            .prune_containers(None::<PruneContainersOptions<String>>)
            .await
            .map_err(|e| {
                AppError::operation_failed(format!("Failed to prune containers: {}", e))
            })?;

        Ok(PruneResultDto {
            deleted_count: result
                .containers_deleted
                .map(|c| c.len() as u32)
                .unwrap_or(0),
            space_freed: result.space_reclaimed.unwrap_or(0) as u64,
        })
    }
}
