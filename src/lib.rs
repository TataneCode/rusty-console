#![allow(dead_code)]

#[path = "02_application/mod.rs"]
pub mod application;
#[path = "01_domain/mod.rs"]
pub mod domain;
#[path = "03_infrastructure/mod.rs"]
pub mod infrastructure;
#[path = "04_presentation/mod.rs"]
pub mod presentation;

pub mod container;
pub mod docker;
pub mod errors;
pub mod image;
pub mod shared;
pub mod stack;
pub mod ui;
pub mod volume;

pub use application::error::AppError;
pub use container::application::{
    ContainerDto, ContainerLogsDto, ContainerRepository, ContainerService,
};
pub use container::domain::{Container, ContainerState};
pub use docker::DockerClient;
pub use domain::error::DomainError;
pub use image::application::{ImageDto, ImageRepository, ImageService};
pub use image::domain::Image;
pub use shared::{ByteSize, PruneResultDto};
pub use stack::application::traits::StackRepository;
pub use stack::application::{StackDto, StackService};
pub use stack::domain::{Stack, StackName};
pub use volume::application::{VolumeDto, VolumeRepository, VolumeService};
pub use volume::domain::Volume;
