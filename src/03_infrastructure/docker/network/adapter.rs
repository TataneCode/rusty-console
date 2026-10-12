use crate::application::error::AppError;
use crate::application::network::NetworkRepository;
use crate::application::shared::PruneResultDto;
use crate::domain::network::Network;
use crate::infrastructure::docker::client::DockerClient;
use crate::infrastructure::docker::network::mapper::NetworkInfraMapper;
use crate::infrastructure::error::InfraError;
use async_trait::async_trait;
use bollard::models::NetworkDisconnectRequest;

pub struct NetworkAdapter {
    docker: DockerClient,
}

impl NetworkAdapter {
    pub fn new(docker: DockerClient) -> Self {
        NetworkAdapter { docker }
    }
}

#[async_trait]
impl NetworkRepository for NetworkAdapter {
    async fn get_all(&self) -> Result<Vec<Network>, AppError> {
        let networks = self
            .docker
            .inner()
            .list_networks(None::<bollard::query_parameters::ListNetworksOptions>)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        // `list_networks` does not reliably fill `containers` or IPAM details,
        // so each network is enriched with an inspect call. A failed inspect
        // (e.g. the network vanished between list and inspect) falls back to
        // the list-only mapping instead of failing the whole screen.
        let mut result = Vec::with_capacity(networks.len());
        for network in &networks {
            match network.name.as_deref() {
                Some(name) => match self.inspect(name).await {
                    Ok(enriched) => result.push(enriched),
                    Err(_) => {
                        if let Some(mapped) = NetworkInfraMapper::from_docker(network) {
                            result.push(mapped);
                        }
                    }
                },
                None => continue,
            }
        }

        Ok(result)
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Network>, AppError> {
        let networks = self.get_all().await?;
        Ok(networks.into_iter().find(|n| n.name() == name))
    }

    async fn inspect(&self, name: &str) -> Result<Network, AppError> {
        let inspect = self
            .docker
            .inner()
            .inspect_network(name, None::<bollard::query_parameters::InspectNetworkOptions>)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        NetworkInfraMapper::from_inspect(&inspect)
            .ok_or_else(|| AppError::not_found(format!("Network {name} not found")))
    }

    async fn delete(&self, name: &str) -> Result<(), AppError> {
        self.docker
            .inner()
            .remove_network(name)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)
    }

    async fn prune(&self) -> Result<PruneResultDto, AppError> {
        let result = self
            .docker
            .inner()
            .prune_networks(None::<bollard::query_parameters::PruneNetworksOptions>)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)?;

        Ok(PruneResultDto {
            deleted_count: result
                .networks_deleted
                .map(|networks| networks.len() as u32)
                .unwrap_or(0),
            space_freed: 0,
        })
    }

    async fn disconnect(
        &self,
        network: &str,
        container: &str,
        force: bool,
    ) -> Result<(), AppError> {
        let request = NetworkDisconnectRequest {
            container: container.to_string(),
            force: Some(force),
        };

        self.docker
            .inner()
            .disconnect_network(network, request)
            .await
            .map_err(InfraError::from)
            .map_err(AppError::from)
    }
}
