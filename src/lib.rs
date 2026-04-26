#![allow(dead_code)]

#[path = "02_application/mod.rs"]
pub mod application;
#[path = "01_domain/mod.rs"]
pub mod domain;
#[path = "03_infrastructure/mod.rs"]
pub mod infrastructure;
#[path = "04_presentation/mod.rs"]
pub mod presentation;

mod shared;

pub use application::container::{
    ContainerDto, ContainerLogsDto, ContainerRepository, ContainerService,
};
pub use application::error::AppError;
pub use application::image::{ImageDto, ImageRepository, ImageService};
pub use application::shared::PruneResultDto;
pub use application::stack::traits::StackRepository;
pub use application::stack::{StackContainerDto, StackDto, StackService};
pub use application::volume::{VolumeDto, VolumeRepository, VolumeService};
pub use domain::container::{Container, ContainerState};
pub use domain::error::DomainError;
pub use domain::image::Image;
pub use domain::shared::ByteSize;
pub use domain::stack::{Stack, StackContainer, StackContainerState, StackName};
pub use domain::volume::Volume;
pub use infrastructure::docker::client::DockerClient;
pub use presentation::tui::app::App;
