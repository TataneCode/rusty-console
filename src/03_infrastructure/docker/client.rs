use crate::infrastructure::error::InfraError;
use bollard::Docker;

pub const DEFAULT_DOCKER_SOCKET_HOST: &str = "unix:///var/run/docker.sock";

pub fn create_docker_client() -> Result<Docker, InfraError> {
    Docker::connect_with_socket_defaults()
        .map_err(|e| InfraError::Connection(format!("Failed to connect to Docker: {}", e)))
}

pub struct DockerClient {
    client: Docker,
}

impl DockerClient {
    pub fn new() -> Result<Self, InfraError> {
        let client = create_docker_client()?;
        Ok(DockerClient { client })
    }

    pub fn inner(&self) -> &Docker {
        &self.client
    }

    pub fn cli_host(&self) -> &'static str {
        DEFAULT_DOCKER_SOCKET_HOST
    }
}

impl Clone for DockerClient {
    fn clone(&self) -> Self {
        DockerClient {
            client: self.client.clone(),
        }
    }
}
