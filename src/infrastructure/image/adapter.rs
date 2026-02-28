use crate::application::{AppError, ImageRepository};
use crate::domain::Image;
use crate::infrastructure::docker::DockerClient;
use crate::infrastructure::image::mapper::ImageInfraMapper;
use async_trait::async_trait;
use bollard::container::ListContainersOptions;
use bollard::image::{ListImagesOptions, RemoveImageOptions};
use std::collections::HashSet;

pub struct ImageAdapter {
    docker: DockerClient,
}

impl ImageAdapter {
    pub fn new(docker: DockerClient) -> Self {
        ImageAdapter { docker }
    }

    async fn get_in_use_image_ids(&self) -> HashSet<String> {
        let options = ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        };

        match self.docker.inner().list_containers(Some(options)).await {
            Ok(containers) => containers
                .into_iter()
                .filter_map(|c| c.image_id)
                .collect(),
            Err(_) => HashSet::new(),
        }
    }
}

#[async_trait]
impl ImageRepository for ImageAdapter {
    async fn get_all(&self) -> Result<Vec<Image>, AppError> {
        let options = ListImagesOptions::<String> {
            all: true,
            ..Default::default()
        };

        let images = self
            .docker
            .inner()
            .list_images(Some(options))
            .await
            .map_err(|e| AppError::repository(e.to_string()))?;

        let in_use_ids = self.get_in_use_image_ids().await;

        Ok(images
            .iter()
            .filter_map(|img| ImageInfraMapper::from_docker(img, &in_use_ids))
            .collect())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Image>, AppError> {
        let images = self.get_all().await?;
        Ok(images.into_iter().find(|i| i.id().as_str() == id || i.id().short() == id))
    }

    async fn delete(&self, id: &str, force: bool) -> Result<(), AppError> {
        let options = RemoveImageOptions {
            force,
            noprune: false,
        };

        self.docker
            .inner()
            .remove_image(id, Some(options), None)
            .await
            .map_err(|e| AppError::operation_failed(format!("Failed to delete image: {}", e)))?;

        Ok(())
    }
}
