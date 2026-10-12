use crate::application::error::AppError;
use crate::application::network::{NetworkDto, NetworkService};
use crate::application::shared::PruneResultDto;

pub struct NetworkActions {
    service: NetworkService,
}

impl NetworkActions {
    pub fn new(service: NetworkService) -> Self {
        NetworkActions { service }
    }

    pub async fn load_networks(&self) -> Result<Vec<NetworkDto>, AppError> {
        self.service.get_all_networks().await
    }

    pub async fn delete_network(&self, name: &str) -> Result<(), AppError> {
        self.service.delete_network(name).await
    }

    pub async fn prune_networks(&self) -> Result<PruneResultDto, AppError> {
        self.service.prune_networks().await
    }

    pub async fn disconnect_container(
        &self,
        network: &str,
        container: &str,
    ) -> Result<(), AppError> {
        self.service.disconnect_container(network, container, false).await
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkActions;
    use crate::application::network::traits::MockNetworkRepository;
    use crate::application::network::NetworkService;
    use crate::application::shared::PruneResultDto;
    use crate::domain::network::{Network, NetworkContainer, NetworkId, NetworkScope};
    use std::sync::Arc;

    fn make_network() -> Network {
        Network::new(
            NetworkId::new("net-1").unwrap(),
            "app-net",
            "bridge",
            NetworkScope::Local,
        )
        .with_containers(vec![NetworkContainer::new("web", "172.20.0.2")])
    }

    #[tokio::test]
    async fn test_network_actions_delegate_all_operations() {
        let mut mock = MockNetworkRepository::new();
        mock.expect_get_all().returning(|| Ok(vec![make_network()]));
        mock.expect_delete().returning(|_| Ok(()));
        mock.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 1,
                space_freed: 0,
            })
        });
        mock.expect_disconnect()
            .withf(|network, container, force| {
                network == "app-net" && container == "web" && !force
            })
            .returning(|_, _, _| Ok(()));

        let actions = NetworkActions::new(NetworkService::new(Arc::new(mock)));
        let networks = actions.load_networks().await.unwrap();

        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].container_count, 1);
        assert!(actions.delete_network(&networks[0].name).await.is_ok());
        assert_eq!(actions.prune_networks().await.unwrap().deleted_count, 1);
        assert!(
            actions
                .disconnect_container("app-net", "web")
                .await
                .is_ok()
        );
    }
}
