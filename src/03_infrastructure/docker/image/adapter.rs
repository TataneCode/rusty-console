use crate::application::error::AppError;
use crate::application::image::dto::{
    ImagePullEvent, ImagePullOutcome, ImagePullProgress, ImagePullSubscription,
};
use crate::application::image::ImageRepository;
use crate::application::shared::PruneResultDto;
use crate::domain::image::Image;
use crate::infrastructure::docker::client::DockerClient;
use crate::infrastructure::docker::image::mapper::ImageInfraMapper;
use crate::infrastructure::error::InfraError;
use async_trait::async_trait;
use bollard::query_parameters::ListContainersOptions;
use bollard::query_parameters::{
    CreateImageOptions, ListImagesOptions, PruneImagesOptions, RemoveImageOptions,
};
use futures_util::StreamExt;
use std::collections::{BTreeSet, HashMap, HashSet};
use tokio::sync::mpsc;

use crate::infrastructure::docker::image::pull::{pull_operation_error, RegistryCredentials};

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

pub(crate) fn unique_image_refs(image_refs: &[String]) -> Vec<String> {
    image_refs
        .iter()
        .filter(|image| !image.trim().is_empty() && image.as_str() != "unknown")
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn local_repo_digest(image_ref: &str, repo_digests: &[String]) -> Option<String> {
    let prefix = format!("{image_ref}@");
    repo_digests
        .iter()
        .find(|digest| digest.starts_with(&prefix))
        .map(|digest| digest[prefix.len()..].to_string())
}

pub struct ImageAdapter {
    docker: DockerClient,
}

impl ImageAdapter {
    pub fn new(docker: DockerClient) -> Self {
        ImageAdapter { docker }
    }

    async fn get_in_use_image_ids(&self) -> Result<HashSet<String>, AppError> {
        let options = ListContainersOptions {
            all: true,
            ..Default::default()
        };

        let containers = self
            .docker
            .inner()
            .list_containers(Some(options))
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        Ok(containers.into_iter().filter_map(|c| c.image_id).collect())
    }
}

#[async_trait]
impl ImageRepository for ImageAdapter {
    async fn get_all(&self) -> Result<Vec<Image>, AppError> {
        let options = ListImagesOptions {
            all: true,
            ..Default::default()
        };

        let images = self
            .docker
            .inner()
            .list_images(Some(options))
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        let in_use_ids = self.get_in_use_image_ids().await?;

        Ok(images
            .iter()
            .filter_map(|img| ImageInfraMapper::from_docker(img, &in_use_ids))
            .collect())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Image>, AppError> {
        let images = self.get_all().await?;
        Ok(images
            .into_iter()
            .find(|i| i.id().as_str() == id || i.id().short() == id))
    }

    async fn delete(&self, id: &str, force: bool) -> Result<(), AppError> {
        let options = RemoveImageOptions {
            force,
            noprune: false,
            platforms: None,
        };

        self.docker
            .inner()
            .remove_image(id, Some(options), None)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn local_digest(&self, image_ref: &str) -> Result<Option<String>, AppError> {
        let inspect = self
            .docker
            .inner()
            .inspect_image(image_ref)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;
        Ok(inspect
            .repo_digests
            .as_deref()
            .and_then(|digests| local_repo_digest(image_ref, digests)))
    }

    async fn registry_digest(&self, image_ref: &str) -> Result<Option<String>, AppError> {
        let credentials = RegistryCredentials::load();
        let credentials = credentials.for_image(image_ref);
        let distribution = self
            .docker
            .inner()
            .inspect_registry_image(image_ref, credentials)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;
        Ok(distribution.descriptor.digest)
    }

    async fn pull_images(&self, image_refs: &[String]) -> Result<ImagePullSubscription, AppError> {
        let images = unique_image_refs(image_refs);
        let (sender, receiver) = mpsc::channel(PULL_EVENT_BUFFER);
        let docker = self.docker.clone();
        let task = tokio::spawn(async move {
            let credentials = RegistryCredentials::load();
            let mut outcome = ImagePullOutcome::empty();
            for image in images {
                let mut layers: HashMap<String, (u64, u64)> = HashMap::new();
                let options = CreateImageOptions {
                    from_image: Some(image.clone()),
                    ..Default::default()
                };
                let image_credentials = credentials.for_image(&image);
                let mut stream =
                    docker
                        .inner()
                        .create_image(Some(options), None, image_credentials);
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
                            pull_error = Some(pull_operation_error(
                                "pull",
                                &image,
                                AppError::from(InfraError::from(error)),
                            ));
                            break;
                        }
                    }
                }
                if let Some(error) = pull_error {
                    let _ = sender.send(ImagePullEvent::Completed(Err(error))).await;
                    return;
                }
                outcome.pulled.push(image);
            }
            let _ = sender.send(ImagePullEvent::Completed(Ok(outcome))).await;
        });
        Ok(ImagePullSubscription::new(receiver, vec![task]))
    }

    async fn prune(&self) -> Result<PruneResultDto, AppError> {
        let mut filters = std::collections::HashMap::new();
        filters.insert("dangling".to_string(), vec!["true".to_string()]);
        let options = PruneImagesOptions {
            filters: Some(filters),
        };

        let result = self
            .docker
            .inner()
            .prune_images(Some(options))
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        Ok(PruneResultDto {
            deleted_count: result.images_deleted.map(|i| i.len() as u32).unwrap_or(0),
            space_freed: result.space_reclaimed.unwrap_or(0) as u64,
        })
    }
}
