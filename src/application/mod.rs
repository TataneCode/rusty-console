pub mod container;
pub mod error;
pub mod image;
pub mod volume;

pub use container::{ContainerDto, ContainerLogsDto, ContainerRepository, ContainerService};
pub use error::AppError;
pub use image::{ImageDto, ImageRepository, ImageService};
pub use volume::{VolumeDto, VolumeRepository, VolumeService};

#[derive(Debug, Clone)]
pub struct PruneResultDto {
    pub deleted_count: u32,
    pub space_freed: u64,
}
