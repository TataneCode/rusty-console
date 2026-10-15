use crate::application::error::AppError;
use crate::application::stack::dto::{ImagePullEvent, ImagePullProgress, ImagePullSubscription};
use crate::application::stack::traits::StackRepository;
use crate::domain::stack::Stack;
use crate::infrastructure::docker::client::DockerClient;
use crate::infrastructure::docker::stack::mapper::StackInfraMapper;
use crate::infrastructure::error::InfraError;
use async_trait::async_trait;
use bollard::query_parameters::{
    CreateImageOptions, ListContainersOptions, RemoveContainerOptions, StartContainerOptions,
    StopContainerOptions,
};
use futures_util::StreamExt;
use std::collections::{BTreeSet, HashMap};
use tokio::sync::mpsc;

const PULL_EVENT_BUFFER: usize = 128;

fn aggregate_layer_progress(
    layers: &HashMap<String, (u64, u64)>,
    image: &str,
) -> ImagePullProgress {
    let (downloaded_bytes, total_bytes) = layers
        .values()
        .fold((0, 0), |(current_sum, total_sum), (current, total)| {
            (current_sum + current, total_sum + total)
        });
    ImagePullProgress {
        image: image.to_string(),
        downloaded_bytes,
        total_bytes,
    }
}

fn track_layer_progress(
    layers: &mut HashMap<String, (u64, u64)>,
    info: &bollard::models::CreateImageInfo,
) -> Option<u64> {
    let detail = info.progress_detail.as_ref()?;
    let (Some(current), Some(total)) = (detail.current, detail.total) else {
        return None;
    };
    if total <= 0 || current < 0 {
        return None;
    }
    let layer_id = info
        .id
        .clone()
        .unwrap_or_else(|| info.status.clone().unwrap_or_default());
    layers.insert(layer_id, (current.unsigned_abs(), total.unsigned_abs()));
    Some(current.unsigned_abs())
}

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
                .start_container(id, None::<StartContainerOptions>)
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
                .stop_container(
                    id,
                    Some(StopContainerOptions {
                        t: Some(10),
                        ..Default::default()
                    }),
                )
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

    async fn pull_images(&self, image_refs: &[String]) -> Result<ImagePullSubscription, AppError> {
        let images = unique_image_refs(image_refs);
        let (sender, receiver) = mpsc::channel(PULL_EVENT_BUFFER);
        let docker = self.docker.clone();

        let task = tokio::spawn(async move {
            for image in images {
                let mut layers: HashMap<String, (u64, u64)> = HashMap::new();
                let options = CreateImageOptions {
                    from_image: Some(image.clone()),
                    ..Default::default()
                };
                let mut stream = docker.inner().create_image(Some(options), None, None);

                let mut pull_error: Option<AppError> = None;
                while let Some(item) = stream.next().await {
                    match item {
                        Ok(info) => {
                            if let Some(error_detail) = &info.error_detail {
                                pull_error = Some(AppError::operation_failed(format!(
                                    "Failed to pull image '{image}': {}",
                                    error_detail.message.clone().unwrap_or_default()
                                )));
                                break;
                            }
                            if track_layer_progress(&mut layers, &info).is_some() {
                                let progress = aggregate_layer_progress(&layers, &image);
                                if sender
                                    .send(ImagePullEvent::Progress(progress))
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                            }
                        }
                        Err(error) => {
                            pull_error = Some(AppError::operation_failed(format!(
                                "Failed to pull image '{image}': {}",
                                AppError::from(InfraError::from(error))
                            )));
                            break;
                        }
                    }
                }

                if let Some(error) = pull_error {
                    let _ = sender.send(ImagePullEvent::Completed(Err(error))).await;
                    return;
                }
            }

            let _ = sender.send(ImagePullEvent::Completed(Ok(()))).await;
        });

        Ok(ImagePullSubscription::new(receiver, vec![task]))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        aggregate_layer_progress, container_operation_error, track_layer_progress,
        unique_image_refs,
    };
    use crate::application::error::AppError;
    use crate::infrastructure::error::InfraError;
    use bollard::models::{CreateImageInfo, ProgressDetail};
    use std::collections::HashMap;

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

    fn make_progress_info(id: &str, current: i64, total: i64) -> CreateImageInfo {
        CreateImageInfo {
            id: Some(id.to_string()),
            status: Some("Downloading".to_string()),
            progress_detail: Some(ProgressDetail {
                current: Some(current),
                total: Some(total),
            }),
            error_detail: None,
        }
    }

    #[test]
    fn track_layer_progress_stores_layer_bytes() {
        let mut layers = HashMap::new();

        assert_eq!(
            track_layer_progress(&mut layers, &make_progress_info("a", 100, 400)),
            Some(100)
        );
        assert_eq!(
            track_layer_progress(&mut layers, &make_progress_info("b", 50, 100)),
            Some(50)
        );
        assert_eq!(
            track_layer_progress(&mut layers, &make_progress_info("a", 300, 400)),
            Some(300)
        );

        assert_eq!(layers.len(), 2);
        assert_eq!(layers.get("a"), Some(&(300, 400)));
        assert_eq!(layers.get("b"), Some(&(50, 100)));
    }

    #[test]
    fn track_layer_progress_ignores_events_without_detail() {
        let mut layers = HashMap::new();
        let info = CreateImageInfo {
            id: Some("a".to_string()),
            status: Some("Pull completed".to_string()),
            progress_detail: None,
            error_detail: None,
        };

        assert_eq!(track_layer_progress(&mut layers, &info), None);
        assert!(layers.is_empty());
    }

    #[test]
    fn aggregate_layer_progress_sums_layers() {
        let mut layers = HashMap::new();
        layers.insert("a".to_string(), (300, 400));
        layers.insert("b".to_string(), (50, 100));

        let progress = aggregate_layer_progress(&layers, "nginx:latest");

        assert_eq!(progress.image, "nginx:latest");
        assert_eq!(progress.downloaded_bytes, 350);
        assert_eq!(progress.total_bytes, 500);
        assert_eq!(progress.percent(), 70);
    }

    #[test]
    fn aggregate_layer_progress_empty_means_zero_percent() {
        let layers: HashMap<String, (u64, u64)> = HashMap::new();
        let progress = aggregate_layer_progress(&layers, "nginx:latest");

        assert_eq!(progress.downloaded_bytes, 0);
        assert_eq!(progress.total_bytes, 0);
        assert_eq!(progress.percent(), 0);
    }
}
