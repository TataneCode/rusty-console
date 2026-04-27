use crate::application::error::AppError;
use crate::application::volume::VolumeRepository;
use crate::domain::volume::Volume;
use crate::infrastructure::docker::client::DockerClient;
use crate::infrastructure::docker::volume::mapper::VolumeInfraMapper;
use crate::infrastructure::error::InfraError;
use crate::shared::PruneResultDto;
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

    async fn get_volume_sizes(&self) -> Result<HashMap<String, i64>, AppError> {
        let response = self
            .docker
            .inner()
            .df()
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        Ok(response
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(|v| {
                let name = v.name;
                let size = v.usage_data.as_ref().map(|u| u.size).unwrap_or(-1);
                (name, size)
            })
            .collect())
    }

    async fn get_in_use_volumes(&self) -> Result<Vec<String>, AppError> {
        let options = ListContainersOptions::<String> {
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

        Ok(containers
            .into_iter()
            .flat_map(|c| {
                c.mounts
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|m| m.name)
            })
            .collect())
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
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        let size_map = self.get_volume_sizes().await?;
        let in_use_volumes = self.get_in_use_volumes().await?;

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
            .map_err(InfraError::from)
            .map_err(AppError::from)
    }

    async fn prune(&self) -> Result<PruneResultDto, AppError> {
        let result = self
            .docker
            .inner()
            .prune_volumes(None::<bollard::volume::PruneVolumesOptions<String>>)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        Ok(PruneResultDto {
            deleted_count: result.volumes_deleted.map(|v| v.len() as u32).unwrap_or(0),
            space_freed: result.space_reclaimed.unwrap_or(0) as u64,
        })
    }
}
