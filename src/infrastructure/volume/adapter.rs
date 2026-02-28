use crate::application::{AppError, VolumeRepository};
use crate::domain::Volume;
use crate::infrastructure::docker::DockerClient;
use crate::infrastructure::volume::mapper::VolumeInfraMapper;
use async_trait::async_trait;
use bollard::container::ListContainersOptions;
use bollard::volume::{ListVolumesOptions, RemoveVolumeOptions};
use std::collections::HashMap;

pub struct VolumeAdapter {
    docker: DockerClient,
}

impl VolumeAdapter {
    pub fn new(docker: DockerClient) -> Self {
        VolumeAdapter { docker }
    }

    async fn get_volume_sizes(&self) -> HashMap<String, i64> {
        match self.docker.inner().df().await {
            Ok(response) => response
                .volumes
                .unwrap_or_default()
                .into_iter()
                .filter_map(|v| {
                    let name = v.name;
                    let size = v.usage_data.as_ref().map(|u| u.size).unwrap_or(-1);
                    Some((name, size))
                })
                .collect(),
            Err(_) => HashMap::new(),
        }
    }

    async fn get_in_use_volumes(&self) -> Vec<String> {
        let options = ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        };

        match self.docker.inner().list_containers(Some(options)).await {
            Ok(containers) => containers
                .into_iter()
                .flat_map(|c| {
                    c.mounts
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|m| m.name)
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

#[async_trait]
impl VolumeRepository for VolumeAdapter {
    async fn get_all(&self) -> Result<Vec<Volume>, AppError> {
        let volumes_response = self
            .docker
            .inner()
            .list_volumes(None::<ListVolumesOptions<String>>)
            .await
            .map_err(|e| AppError::repository(e.to_string()))?;

        let size_map = self.get_volume_sizes().await;
        let in_use_volumes = self.get_in_use_volumes().await;

        let volumes = volumes_response
            .volumes
            .unwrap_or_default()
            .iter()
            .filter_map(|v| VolumeInfraMapper::from_docker(v, &size_map, &in_use_volumes))
            .collect();

        Ok(volumes)
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Volume>, AppError> {
        let volumes = self.get_all().await?;
        Ok(volumes.into_iter().find(|v| v.name() == name))
    }

    async fn delete(&self, name: &str) -> Result<(), AppError> {
        let options = RemoveVolumeOptions { force: false };

        self.docker
            .inner()
            .remove_volume(name, Some(options))
            .await
            .map_err(|e| AppError::operation_failed(format!("Failed to delete volume: {}", e)))
    }
}
