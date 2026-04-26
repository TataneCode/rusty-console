use crate::domain::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContainerId(String);

impl ContainerId {
    pub fn new(id: impl Into<String>) -> Result<Self, DomainError> {
        let id = id.into();
        if id.is_empty() {
            return Err(DomainError::InvalidContainerId(
                "Container ID cannot be empty".to_string(),
            ));
        }
        Ok(ContainerId(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn short(&self) -> &str {
        if self.0.len() > 12 {
            &self.0[..12]
        } else {
            &self.0
        }
    }
}

impl std::fmt::Display for ContainerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortMapping {
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub protocol: String,
}

impl PortMapping {
    pub fn new(private_port: u16, public_port: Option<u16>, protocol: impl Into<String>) -> Self {
        PortMapping {
            private_port,
            public_port,
            protocol: protocol.into(),
        }
    }
}

impl std::fmt::Display for PortMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.public_port {
            Some(public) => write!(f, "{}:{}/{}", public, self.private_port, self.protocol),
            None => write!(f, "{}/{}", self.private_port, self.protocol),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkInfo {
    pub name: String,
    pub ip_address: String,
}

impl NetworkInfo {
    pub fn new(name: impl Into<String>, ip_address: impl Into<String>) -> Self {
        NetworkInfo {
            name: name.into(),
            ip_address: ip_address.into(),
        }
    }
}

impl std::fmt::Display for NetworkInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ip_address.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{} ({})", self.name, self.ip_address)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountInfo {
    pub source: String,
    pub destination: String,
    pub mode: String,
}

impl MountInfo {
    pub fn new(
        source: impl Into<String>,
        destination: impl Into<String>,
        mode: impl Into<String>,
    ) -> Self {
        MountInfo {
            source: source.into(),
            destination: destination.into(),
            mode: mode.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_id_valid() {
        let id = ContainerId::new("abc123").unwrap();
        assert_eq!(id.as_str(), "abc123");
    }

    #[test]
    fn test_container_id_empty() {
        let result = ContainerId::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_container_id_short() {
        let id = ContainerId::new("abcdef1234567890").unwrap();
        assert_eq!(id.short(), "abcdef123456");
    }

    #[test]
    fn test_port_mapping_display() {
        let port = PortMapping::new(80, Some(8080), "tcp");
        assert_eq!(format!("{}", port), "8080:80/tcp");

        let port_no_public = PortMapping::new(80, None, "tcp");
        assert_eq!(format!("{}", port_no_public), "80/tcp");
    }
}
