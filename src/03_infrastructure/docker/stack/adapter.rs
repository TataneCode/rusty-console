use crate::application::error::AppError;
use crate::application::stack::traits::StackRepository;
use crate::domain::stack::Stack;
use crate::infrastructure::docker::client::DockerClient;
use crate::infrastructure::docker::stack::mapper::StackInfraMapper;
use crate::infrastructure::error::InfraError;
use async_trait::async_trait;
use bollard::container::{
    ListContainersOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use std::collections::HashMap;

pub struct StackAdapter {
    docker: DockerClient,
}

fn container_operation_error(operation: &str, id: &str, err: InfraError) -> AppError {
    match err {
        InfraError::Connection(message) => {
            AppError::connection(format!("Failed to {operation} container '{id}': {message}"))
        }
        InfraError::Docker(message) | InfraError::Serialization(message) => {
            AppError::repository(format!("Failed to {operation} container '{id}': {message}"))
        }
    }
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
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        Ok(StackInfraMapper::group_into_stacks(summaries))
    }

    async fn start_all(&self, container_ids: &[String]) -> Result<(), AppError> {
        for id in container_ids {
            self.docker
                .inner()
                .start_container(id, None::<StartContainerOptions<String>>)
                .await
                .map_err(InfraError::from)
                .map_err(|err| container_operation_error("start", id, err))?;
        }
        Ok(())
    }

    async fn stop_all(&self, container_ids: &[String]) -> Result<(), AppError> {
        for id in container_ids {
            self.docker
                .inner()
                .stop_container(id, Some(StopContainerOptions { t: 10 }))
                .await
                .map_err(InfraError::from)
                .map_err(|err| container_operation_error("stop", id, err))?;
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
                .map_err(InfraError::from)
                .map_err(|err| container_operation_error("remove", id, err))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::container_operation_error;
    use crate::application::error::AppError;
    use crate::infrastructure::error::InfraError;

    #[test]
    fn container_operation_error_keeps_container_id_for_repository_errors() {
        let err =
            container_operation_error("start", "abc123", InfraError::Docker("timeout".into()));

        assert!(matches!(err, AppError::Repository(_)));
        assert_eq!(
            err.to_string(),
            "Repository error: Failed to start container 'abc123': timeout"
        );
    }

    #[test]
    fn container_operation_error_keeps_container_id_for_connection_errors() {
        let err =
            container_operation_error("stop", "xyz789", InfraError::Connection("refused".into()));

        assert!(matches!(err, AppError::Connection(_)));
        assert_eq!(
            err.to_string(),
            "Connection error: Failed to stop container 'xyz789': refused"
        );
    }
}
