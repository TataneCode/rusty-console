use rusty_console::application::container::ContainerService;
use rusty_console::application::image::ImageService;
use rusty_console::application::stack::StackService;
use rusty_console::application::volume::VolumeService;
use rusty_console::infrastructure::docker::client::DockerClient;
use rusty_console::infrastructure::docker::container::ContainerAdapter;
use rusty_console::infrastructure::docker::image::ImageAdapter;
use rusty_console::infrastructure::docker::stack::StackAdapter;
use rusty_console::infrastructure::docker::volume::VolumeAdapter;
use rusty_console::presentation::tui::app::{App, ExecCommandConfig};
use rusty_console::presentation::tui::container::ContainerActions;
use rusty_console::presentation::tui::image::ImageActions;
use rusty_console::presentation::tui::stack::StackActions;
use rusty_console::presentation::tui::volume::VolumeActions;
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
        ExecCommandConfig::new(docker.cli_host()),
    );
    app.run().await?;
    Ok(())
}
