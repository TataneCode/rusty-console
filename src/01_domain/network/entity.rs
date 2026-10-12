use chrono::{DateTime, Utc};

use super::value_objects::{NetworkId, NetworkScope};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkContainer {
    name: String,
    ip: String,
}

impl NetworkContainer {
    pub fn new(name: impl Into<String>, ip: impl Into<String>) -> Self {
        NetworkContainer {
            name: name.into(),
            ip: ip.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ip(&self) -> &str {
        &self.ip
    }
}

#[derive(Debug, Clone)]
pub struct Network {
    id: NetworkId,
    name: String,
    driver: String,
    scope: NetworkScope,
    internal: bool,
    attachable: bool,
    subnet: Option<String>,
    gateway: Option<String>,
    created: Option<DateTime<Utc>>,
    builtin: bool,
    containers: Vec<NetworkContainer>,
}

impl Network {
    pub fn new(
        id: NetworkId,
        name: impl Into<String>,
        driver: impl Into<String>,
        scope: NetworkScope,
    ) -> Self {
        Network {
            id,
            name: name.into(),
            driver: driver.into(),
            scope,
            internal: false,
            attachable: false,
            subnet: None,
            gateway: None,
            created: None,
            builtin: false,
            containers: Vec::new(),
        }
    }

    pub fn with_internal(mut self, internal: bool) -> Self {
        self.internal = internal;
        self
    }

    pub fn with_attachable(mut self, attachable: bool) -> Self {
        self.attachable = attachable;
        self
    }

    pub fn with_subnet(mut self, subnet: String) -> Self {
        self.subnet = Some(subnet);
        self
    }

    pub fn with_gateway(mut self, gateway: String) -> Self {
        self.gateway = Some(gateway);
        self
    }

    pub fn with_created(mut self, created: DateTime<Utc>) -> Self {
        self.created = Some(created);
        self
    }

    pub fn with_builtin(mut self, builtin: bool) -> Self {
        self.builtin = builtin;
        self
    }

    pub fn with_containers(mut self, containers: Vec<NetworkContainer>) -> Self {
        self.containers = containers;
        self
    }

    // Getters
    pub fn id(&self) -> &NetworkId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn driver(&self) -> &str {
        &self.driver
    }

    pub fn scope(&self) -> NetworkScope {
        self.scope
    }

    pub fn internal(&self) -> bool {
        self.internal
    }

    pub fn attachable(&self) -> bool {
        self.attachable
    }

    pub fn subnet(&self) -> Option<&str> {
        self.subnet.as_deref()
    }

    pub fn gateway(&self) -> Option<&str> {
        self.gateway.as_deref()
    }

    pub fn created(&self) -> Option<DateTime<Utc>> {
        self.created
    }

    pub fn builtin(&self) -> bool {
        self.builtin
    }

    pub fn containers(&self) -> &[NetworkContainer] {
        &self.containers
    }

    // Business logic
    pub fn can_be_deleted(&self) -> bool {
        !self.builtin && self.containers.is_empty()
    }

    pub fn can_disconnect(&self) -> bool {
        !self.containers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_network() -> Network {
        Network::new(
            NetworkId::new("net-1").unwrap(),
            "my-network",
            "bridge",
            NetworkScope::Local,
        )
    }

    #[test]
    fn test_builder_with_optional_fields() {
        let network = make_network()
            .with_internal(true)
            .with_attachable(true)
            .with_subnet("172.20.0.0/16".to_string())
            .with_gateway("172.20.0.1".to_string())
            .with_builtin(false)
            .with_containers(vec![NetworkContainer::new("web", "172.20.0.2")]);

        assert!(network.internal());
        assert!(network.attachable());
        assert_eq!(network.subnet(), Some("172.20.0.0/16"));
        assert_eq!(network.gateway(), Some("172.20.0.1"));
        assert_eq!(network.containers().len(), 1);
        assert_eq!(network.containers()[0].name(), "web");
        assert_eq!(network.containers()[0].ip(), "172.20.0.2");
    }

    #[test]
    fn test_can_be_deleted_when_user_defined_and_unused() {
        let network = make_network();
        assert!(network.can_be_deleted());
    }

    #[test]
    fn test_cannot_be_deleted_when_builtin() {
        let network = make_network().with_builtin(true);
        assert!(!network.can_be_deleted());
    }

    #[test]
    fn test_cannot_be_deleted_when_containers_connected() {
        let network = make_network()
            .with_containers(vec![NetworkContainer::new("web", "172.20.0.2")]);
        assert!(!network.can_be_deleted());
    }

    #[test]
    fn test_can_disconnect_only_with_connected_containers() {
        let network = make_network();
        assert!(!network.can_disconnect());

        let network =
            network.with_containers(vec![NetworkContainer::new("web", "172.20.0.2")]);
        assert!(network.can_disconnect());
    }

    #[test]
    fn test_getters() {
        let network = make_network();
        assert_eq!(network.id().as_str(), "net-1");
        assert_eq!(network.name(), "my-network");
        assert_eq!(network.driver(), "bridge");
        assert_eq!(network.scope(), NetworkScope::Local);
        assert!(!network.builtin());
        assert!(network.created().is_none());
    }
}
