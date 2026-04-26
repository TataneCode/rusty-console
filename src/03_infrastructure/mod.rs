//! Transitional bridge for the target infrastructure layer.
//!
//! Boundary rules for this layer:
//! - Implement application ports and talk to external systems.
//! - Keep Docker/Bollard-specific code here.
//! - Depend inward on application and domain code, never the other way around.
//! - Presentation-specific concerns do not belong here.

pub mod error {
    pub use crate::errors::infrastructure::InfraError;
}

pub mod docker {
    pub mod client {
        pub use crate::docker::{create_docker_client, DockerClient};
    }

    pub mod container {
        pub use crate::container::infrastructure::adapter::ContainerAdapter;
        pub use crate::container::infrastructure::mapper::ContainerInfraMapper;
    }

    pub mod image {
        pub use crate::image::infrastructure::adapter::ImageAdapter;
        pub use crate::image::infrastructure::mapper::ImageInfraMapper;
    }

    pub mod volume {
        pub use crate::volume::infrastructure::adapter::VolumeAdapter;
        pub use crate::volume::infrastructure::mapper::VolumeInfraMapper;
    }

    pub mod stack {
        pub use crate::stack::infrastructure::adapter::StackAdapter;
        pub use crate::stack::infrastructure::mapper::StackInfraMapper;
    }
}
