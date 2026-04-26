#![allow(unused_imports)]

//! Transitional bridge for infrastructure Docker integration.
//!
//! `client.rs` is now the canonical home of the shared Docker client wrapper.
//! Feature-specific Docker adapters will move under this directory incrementally.

pub mod client;

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
