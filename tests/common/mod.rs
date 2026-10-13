use rusty_console::application::container::ContainerService;
use rusty_console::application::image::ImageService;
use rusty_console::application::network::NetworkService;
use rusty_console::application::stack::StackService;
use rusty_console::application::volume::VolumeService;
use rusty_console::infrastructure::docker::client::DockerClient;
use rusty_console::infrastructure::docker::container::ContainerAdapter;
use rusty_console::infrastructure::docker::image::ImageAdapter;
use rusty_console::infrastructure::docker::network::NetworkAdapter;
use rusty_console::infrastructure::docker::stack::StackAdapter;
use rusty_console::infrastructure::docker::volume::VolumeAdapter;
use rusty_console::presentation::tui::app::App;
use rusty_console::presentation::tui::container::ContainerActions;
use rusty_console::presentation::tui::exec::ExecCommandConfig;
use rusty_console::presentation::tui::image::ImageActions;
use rusty_console::presentation::tui::network::NetworkActions;
use rusty_console::presentation::tui::stack::StackActions;
use rusty_console::presentation::tui::volume::VolumeActions;
use std::sync::Arc;

// Mirrors the manual DI wiring of main.rs. The bollard client connects
// lazily, so no Docker daemon is required: rendering performs no I/O.
pub fn build_app() -> Option<App> {
    let Ok(docker) = DockerClient::new() else {
        eprintln!("Docker client unavailable, skipping integration test");
        return None;
    };

    let container_adapter = Arc::new(ContainerAdapter::new(docker.clone()));
    let volume_adapter = Arc::new(VolumeAdapter::new(docker.clone()));
    let image_adapter = Arc::new(ImageAdapter::new(docker.clone()));
    let stack_adapter = Arc::new(StackAdapter::new(docker.clone()));
    let network_adapter = Arc::new(NetworkAdapter::new(docker.clone()));

    let container_actions = ContainerActions::new(ContainerService::new(container_adapter));
    let volume_actions = VolumeActions::new(VolumeService::new(volume_adapter));
    let image_actions = ImageActions::new(ImageService::new(image_adapter));
    let stack_actions = StackActions::new(StackService::new(stack_adapter));
    let network_actions = NetworkActions::new(NetworkService::new(network_adapter));

    Some(App::new(
        container_actions,
        volume_actions,
        image_actions,
        stack_actions,
        network_actions,
        ExecCommandConfig::new(docker.cli_host()),
    ))
}
