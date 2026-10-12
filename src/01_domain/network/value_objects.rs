use crate::domain::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkScope {
    Local,
    Swarm,
    Global,
}

impl NetworkScope {
    pub fn from_docker(scope: &str) -> Self {
        match scope {
            "swarm" => NetworkScope::Swarm,
            "global" => NetworkScope::Global,
            _ => NetworkScope::Local,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            NetworkScope::Local => "local",
            NetworkScope::Swarm => "swarm",
            NetworkScope::Global => "global",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkId(String);

impl NetworkId {
    pub fn new(id: impl Into<String>) -> Result<Self, DomainError> {
        let id = id.into();
        if id.is_empty() {
            return Err(DomainError::InvalidNetwork(
                "Network ID cannot be empty".to_string(),
            ));
        }
        Ok(NetworkId(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NetworkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_id_valid() {
        let id = NetworkId::new("net-abc123").unwrap();
        assert_eq!(id.as_str(), "net-abc123");
    }

    #[test]
    fn test_network_id_empty() {
        let result = NetworkId::new("");
        assert!(matches!(result, Err(DomainError::InvalidNetwork(_))));
    }

    #[test]
    fn test_network_scope_from_docker_values() {
        assert_eq!(NetworkScope::from_docker("local"), NetworkScope::Local);
        assert_eq!(NetworkScope::from_docker("swarm"), NetworkScope::Swarm);
        assert_eq!(NetworkScope::from_docker("global"), NetworkScope::Global);
        assert_eq!(NetworkScope::from_docker("unknown"), NetworkScope::Local);
    }

    #[test]
    fn test_network_scope_display() {
        assert_eq!(NetworkScope::Swarm.as_str(), "swarm");
    }
}
