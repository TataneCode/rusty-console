use crate::application::error::AppError;
use crate::application::stack::traits::StackRepository;
use crate::domain::stack::Stack;
use crate::infrastructure::docker::client::DockerClient;
use crate::infrastructure::docker::stack::mapper::StackInfraMapper;
use async_trait::async_trait;
use bollard::container::{
    ListContainersOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use std::collections::HashMap;

pub struct StackAdapter {
    docker: DockerClient,
}

impl StackAdapter {
    pub fn new(docker: DockerClient) -> Self {
        StackAdapter { docker }
    }
}

#[async_trait]
impl StackRepository for StackAdapter {
    async fn get_all(&self) -> Result<Vec<Stack>, AppError> {
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

        let summaries = self
            .docker
            .inner()
            .list_containers(Some(options))
            .await
            .map_err(|e| AppError::repository(e.to_string()))?;

        Ok(StackInfraMapper::group_into_stacks(summaries))
    }

    async fn start_all(&self, container_ids: &[String]) -> Result<(), AppError> {
        for id in container_ids {
            self.docker
                .inner()
                .start_container(id, None::<StartContainerOptions<String>>)
                .await
                .map_err(|e| {
                    AppError::operation_failed(format!("Failed to start container {}: {}", id, e))
                })?;
        }
        Ok(())
    }

    async fn stop_all(&self, container_ids: &[String]) -> Result<(), AppError> {
        for id in container_ids {
            self.docker
                .inner()
                .stop_container(id, Some(StopContainerOptions { t: 10 }))
                .await
                .map_err(|e| {
                    AppError::operation_failed(format!("Failed to stop container {}: {}", id, e))
                })?;
        }
        Ok(())
    }

    async fn remove_all(&self, container_ids: &[String]) -> Result<(), AppError> {
        for id in container_ids {
            self.docker
                .inner()
                .remove_container(
                    id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await
                .map_err(|e| {
                    AppError::operation_failed(format!("Failed to remove container {}: {}", id, e))
                })?;
        }
        Ok(())
    }
}
