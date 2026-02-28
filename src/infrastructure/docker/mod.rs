use bollard::Docker;
use crate::infrastructure::error::InfraError;

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
}

impl Clone for DockerClient {
    fn clone(&self) -> Self {
        DockerClient {
            client: self.client.clone(),
        }
    }
}
