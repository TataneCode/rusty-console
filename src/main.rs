#![allow(dead_code)]

#[path = "02_application/mod.rs"]
mod application;
#[path = "01_domain/mod.rs"]
mod domain;
#[path = "03_infrastructure/mod.rs"]
mod infrastructure;
#[path = "04_presentation/mod.rs"]
mod presentation;

mod container;
mod docker;
mod errors;
mod image;
mod shared;
mod stack;
mod ui;
mod volume;

use container::application::ContainerService;
use container::infrastructure::adapter::ContainerAdapter;
use container::ui::ContainerActions;
use docker::DockerClient;
use image::application::ImageService;
use image::infrastructure::adapter::ImageAdapter;
use image::ui::ImageActions;
use stack::application::StackService;
use stack::infrastructure::adapter::StackAdapter;
use stack::ui::StackActions;
use ui::app::App;
use volume::application::VolumeService;
use volume::infrastructure::adapter::VolumeAdapter;
use volume::ui::VolumeActions;

use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let docker =
        DockerClient::new().map_err(|e| anyhow::anyhow!("Failed to connect to Docker: {}", e))?;

    let container_adapter = Arc::new(ContainerAdapter::new(docker.clone()));
    let volume_adapter = Arc::new(VolumeAdapter::new(docker.clone()));
    let image_adapter = Arc::new(ImageAdapter::new(docker.clone()));
    let stack_adapter = Arc::new(StackAdapter::new(docker.clone()));

    let container_service = ContainerService::new(container_adapter);
    let volume_service = VolumeService::new(volume_adapter);
    let image_service = ImageService::new(image_adapter);
    let stack_service = StackService::new(stack_adapter);

    let container_actions = ContainerActions::new(container_service);
    let volume_actions = VolumeActions::new(volume_service);
    let image_actions = ImageActions::new(image_service);
    let stack_actions = StackActions::new(stack_service);

    let mut app = App::new(
        container_actions,
        volume_actions,
        image_actions,
        stack_actions,
    );
    app.run().await?;

    Ok(())
}
