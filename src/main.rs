#![allow(dead_code)]

mod application;
mod domain;
mod infrastructure;
mod ui;

use application::{ContainerService, ImageService, VolumeService};
use infrastructure::{ContainerAdapter, DockerClient, ImageAdapter, VolumeAdapter};
use ui::app::App;
use ui::container::ContainerActions;
use ui::image::ImageActions;
use ui::volume::VolumeActions;

use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let docker =
        DockerClient::new().map_err(|e| anyhow::anyhow!("Failed to connect to Docker: {}", e))?;

    let container_adapter = Arc::new(ContainerAdapter::new(docker.clone()));
    let volume_adapter = Arc::new(VolumeAdapter::new(docker.clone()));
    let image_adapter = Arc::new(ImageAdapter::new(docker.clone()));

    let container_service = ContainerService::new(container_adapter);
    let volume_service = VolumeService::new(volume_adapter);
    let image_service = ImageService::new(image_adapter);

    let container_actions = ContainerActions::new(container_service);
    let volume_actions = VolumeActions::new(volume_service);
    let image_actions = ImageActions::new(image_service);

    let mut app = App::new(container_actions, volume_actions, image_actions);
    app.run().await?;

    Ok(())
}
