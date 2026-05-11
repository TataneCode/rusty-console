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
use std::collections::{BTreeSet, HashMap};
use std::io;
use tokio::process::Command;

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

fn unique_image_refs(image_refs: &[String]) -> Vec<String> {
    image_refs
        .iter()
        .filter(|image| !image.trim().is_empty() && image.as_str() != "unknown")
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn image_pull_spawn_error(image: &str, err: &io::Error) -> AppError {
    match err.kind() {
        io::ErrorKind::NotFound => AppError::operation_failed(format!(
            "Docker CLI not found on PATH while pulling image '{image}'. Install the `docker` binary to use stack pull. Raw error: {err}"
        )),
        io::ErrorKind::PermissionDenied => AppError::operation_failed(format!(
            "Docker CLI exists but is not executable while pulling image '{image}'. Check the `docker` binary permissions. Raw error: {err}"
        )),
        _ => AppError::operation_failed(format!(
            "Failed to start `docker pull` for image '{image}'. Raw error: {err}"
        )),
    }
}

fn image_pull_status_error(image: &str, output: &std::process::Output) -> AppError {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let details = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("`docker pull` exited with status {}", output.status)
    };

    AppError::operation_failed(format!("Failed to pull image '{image}': {details}"))
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

    async fn pull_images(&self, image_refs: &[String]) -> Result<(), AppError> {
        for image in unique_image_refs(image_refs) {
            let output = Command::new("docker")
                .args(["--host", self.docker.cli_host(), "pull", image.as_str()])
                .output()
                .await
                .map_err(|err| image_pull_spawn_error(&image, &err))?;

            if !output.status.success() {
                return Err(image_pull_status_error(&image, &output));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        container_operation_error, image_pull_spawn_error, image_pull_status_error,
        unique_image_refs,
    };
    use crate::application::error::AppError;
    use crate::infrastructure::error::InfraError;
    use std::io;
    use std::os::unix::process::ExitStatusExt;

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

    #[test]
    fn unique_image_refs_sorts_and_deduplicates_pullable_images() {
        let unique = unique_image_refs(&[
            "postgres:16".to_string(),
            "nginx:latest".to_string(),
            "postgres:16".to_string(),
            "".to_string(),
            "unknown".to_string(),
        ]);

        assert_eq!(
            unique,
            vec!["nginx:latest".to_string(), "postgres:16".to_string()]
        );
    }

    #[test]
    fn image_pull_spawn_error_explains_missing_docker_cli() {
        let err = image_pull_spawn_error(
            "nginx:latest",
            &io::Error::new(io::ErrorKind::NotFound, "docker"),
        );

        assert!(matches!(err, AppError::OperationFailed(_)));
        assert!(err.to_string().contains("Docker CLI not found"));
    }

    #[test]
    fn image_pull_status_error_uses_stderr_output() {
        let err = image_pull_status_error(
            "nginx:latest",
            &std::process::Output {
                status: std::process::ExitStatus::from_raw(256),
                stdout: Vec::new(),
                stderr: b"pull access denied".to_vec(),
            },
        );

        assert!(matches!(err, AppError::OperationFailed(_)));
        assert!(err.to_string().contains("pull access denied"));
    }
}
