use crate::application::error::AppError;
use crate::application::network::dto::NetworkDto;
use crate::application::network::mapper::NetworkMapper;
use crate::application::network::traits::NetworkRepository;
use crate::application::shared::PruneResultDto;
use std::sync::Arc;

pub struct NetworkService {
    repository: Arc<dyn NetworkRepository>,
}

impl NetworkService {
    pub fn new(repository: Arc<dyn NetworkRepository>) -> Self {
        NetworkService { repository }
    }

    pub async fn get_all_networks(&self) -> Result<Vec<NetworkDto>, AppError> {
        let networks = self.repository.get_all().await?;
        Ok(NetworkMapper::to_dto_list(&networks))
    }

    pub async fn get_network_by_name(&self, name: &str) -> Result<Option<NetworkDto>, AppError> {
        let network = self.repository.get_by_name(name).await?;
        Ok(network.map(|n| NetworkMapper::to_dto(&n)))
    }

    pub async fn inspect_network(&self, name: &str) -> Result<NetworkDto, AppError> {
        let network = self.repository.inspect(name).await?;
        Ok(NetworkMapper::to_dto(&network))
    }

    pub async fn delete_network(&self, name: &str) -> Result<(), AppError> {
        self.repository.delete(name).await
    }

    pub async fn prune_networks(&self) -> Result<PruneResultDto, AppError> {
        self.repository.prune().await
    }

    pub async fn disconnect_container(
        &self,
        network: &str,
        container: &str,
        force: bool,
    ) -> Result<(), AppError> {
        self.repository.disconnect(network, container, force).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::error::AppError;
    use crate::application::network::traits::MockNetworkRepository;
    use crate::application::shared::PruneResultDto;
    use crate::domain::network::{Network, NetworkContainer, NetworkId, NetworkScope};
    use std::sync::Arc;

    fn make_network(id: &str, name: &str) -> Network {
        Network::new(
            NetworkId::new(id).unwrap(),
            name,
            "bridge",
            NetworkScope::Local,
        )
    }

    #[tokio::test]
    async fn test_get_all_networks_returns_dtos() {
        let mut mock = MockNetworkRepository::new();
        mock.expect_get_all().returning(|| {
            Ok(vec![
                make_network("net1", "my-network"),
                make_network("net2", "db-network"),
            ])
        });

        let service = NetworkService::new(Arc::new(mock));
        let result = service.get_all_networks().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "my-network");
        assert_eq!(result[0].driver, "bridge");
        assert_eq!(result[1].name, "db-network");
    }

    #[tokio::test]
    async fn test_get_network_by_name_found() {
        let mut mock = MockNetworkRepository::new();
        mock.expect_get_by_name()
            .withf(|name| name == "my-network")
            .returning(|_| Ok(Some(make_network("net1", "my-network"))));

        let service = NetworkService::new(Arc::new(mock));
        let result = service.get_network_by_name("my-network").await.unwrap();
        assert!(result.is_some());
        let dto = result.unwrap();
        assert_eq!(dto.name, "my-network");
        assert!(dto.can_delete);
    }

    #[tokio::test]
    async fn test_get_network_by_name_not_found() {
        let mut mock = MockNetworkRepository::new();
        mock.expect_get_by_name().returning(|_| Ok(None));

        let service = NetworkService::new(Arc::new(mock));
        let result = service.get_network_by_name("missing").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_inspect_network_returns_enriched_dto() {
        let mut mock = MockNetworkRepository::new();
        mock.expect_inspect()
            .withf(|name| name == "my-network")
            .returning(|_| {
                Ok(make_network("net1", "my-network")
                    .with_containers(vec![NetworkContainer::new("web", "172.20.0.2")]))
            });

        let service = NetworkService::new(Arc::new(mock));
        let dto = service.inspect_network("my-network").await.unwrap();
        assert_eq!(dto.name, "my-network");
        assert_eq!(dto.container_count, 1);
        assert!(!dto.can_delete);
    }

    #[tokio::test]
    async fn test_delete_network_delegates() {
        let mut mock = MockNetworkRepository::new();
        mock.expect_delete()
            .withf(|name| name == "my-network")
            .returning(|_| Ok(()));

        let service = NetworkService::new(Arc::new(mock));
        assert!(service.delete_network("my-network").await.is_ok());
    }

    #[tokio::test]
    async fn test_prune_networks_returns_result() {
        let mut mock = MockNetworkRepository::new();
        mock.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 2,
                space_freed: 0,
            })
        });

        let service = NetworkService::new(Arc::new(mock));
        let result = service.prune_networks().await.unwrap();
        assert_eq!(result.deleted_count, 2);
        assert_eq!(result.space_freed, 0);
    }

    #[tokio::test]
    async fn test_disconnect_container_delegates() {
        let mut mock = MockNetworkRepository::new();
        mock.expect_disconnect()
            .withf(|network, container, force| {
                network == "my-network" && container == "web" && !force
            })
            .returning(|_, _, _| Ok(()));

        let service = NetworkService::new(Arc::new(mock));
        assert!(service
            .disconnect_container("my-network", "web", false)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_get_all_networks_propagates_repository_error() {
        let mut mock = MockNetworkRepository::new();
        mock.expect_get_all()
            .returning(|| Err(AppError::repository("docker down")));

        let service = NetworkService::new(Arc::new(mock));
        let result = service.get_all_networks().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("docker down"));
    }

    #[tokio::test]
    async fn test_disconnect_container_propagates_error() {
        let mut mock = MockNetworkRepository::new();
        mock.expect_disconnect()
            .returning(|_, _, _| Err(AppError::repository("not connected")));

        let service = NetworkService::new(Arc::new(mock));
        let result = service
            .disconnect_container("my-network", "web", false)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not connected"));
    }
}
