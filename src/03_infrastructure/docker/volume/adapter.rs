use crate::application::error::AppError;
use crate::application::volume::VolumeRepository;
use crate::domain::volume::Volume;
use crate::infrastructure::docker::client::DockerClient;
use crate::infrastructure::docker::volume::mapper::VolumeInfraMapper;
use crate::infrastructure::error::InfraError;
use crate::shared::PruneResultDto;
use async_trait::async_trait;
use bollard::container::ListContainersOptions;
use bollard::models::ContainerSummary;
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

    fn volume_links_by_name(containers: Vec<ContainerSummary>) -> HashMap<String, Vec<String>> {
        let mut links: HashMap<String, Vec<String>> = HashMap::new();

        for container in containers {
            let container_name = container
                .names
                .as_ref()
                .and_then(|names| names.first())
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            for volume_name in container
                .mounts
                .unwrap_or_default()
                .into_iter()
                .filter_map(|mount| mount.name)
            {
                let entry = links.entry(volume_name).or_default();
                if !entry.contains(&container_name) {
                    entry.push(container_name.clone());
                }
            }
        }

        for names in links.values_mut() {
            names.sort();
        }

        links
    }

    async fn get_linked_containers_by_volume(
        &self,
    ) -> Result<HashMap<String, Vec<String>>, AppError> {
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

        Ok(Self::volume_links_by_name(containers))
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
        let linked_containers_by_volume = self.get_linked_containers_by_volume().await?;

        let volumes = volumes_response
            .volumes
            .unwrap_or_default()
            .iter()
            .filter_map(|v| {
                VolumeInfraMapper::from_docker(v, &size_map, &linked_containers_by_volume)
            })
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

#[cfg(test)]
mod tests {
    use super::VolumeAdapter;
    use bollard::models::{ContainerSummary, MountPoint};

    fn make_container(name: &str, volumes: &[&str]) -> ContainerSummary {
        ContainerSummary {
            names: Some(vec![name.to_string()]),
            mounts: Some(
                volumes
                    .iter()
                    .map(|volume| MountPoint {
                        name: Some((*volume).to_string()),
                        ..Default::default()
                    })
                    .collect(),
            ),
            ..Default::default()
        }
    }

    #[test]
    fn volume_links_by_name_groups_and_deduplicates_container_names() {
        let links = VolumeAdapter::volume_links_by_name(vec![
            make_container("/worker", &["shared-data", "cache"]),
            make_container("/web", &["shared-data"]),
            make_container("/web", &["shared-data"]),
        ]);

        assert_eq!(
            links.get("shared-data"),
            Some(&vec!["/web".to_string(), "/worker".to_string()])
        );
        assert_eq!(links.get("cache"), Some(&vec!["/worker".to_string()]));
    }
}
