use rusty_console::application::container::{ContainerLogsDto, ContainerService};
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
use rusty_console::presentation::tui::app::{App, Screen};
use rusty_console::presentation::tui::common::resources;
use rusty_console::presentation::tui::container::ContainerActions;
use rusty_console::presentation::tui::exec::ExecCommandConfig;
use rusty_console::presentation::tui::image::ImageActions;
use rusty_console::presentation::tui::network::NetworkActions;
use rusty_console::presentation::tui::stack::StackActions;
use rusty_console::presentation::tui::volume::VolumeActions;
use std::sync::Arc;

// Mirrors the manual DI wiring of main.rs. The bollard client connects
// lazily, so no Docker daemon is required: rendering performs no I/O.
fn build_app() -> Option<App> {
    let Ok(docker) = DockerClient::new() else {
        eprintln!("Docker client unavailable, skipping breadcrumb UI integration test");
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

fn render_app(app: &mut App) -> String {
    let backend = ratatui::backend::TestBackend::new(100, 20);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
}

#[test]
fn main_menu_shows_application_title() {
    let Some(mut app) = build_app() else {
        return;
    };
    assert_eq!(app.screen, Screen::MainMenu);
    let rendered = render_app(&mut app);
    assert!(rendered.contains(resources::APP_TITLE));
}

#[test]
fn container_logs_screen_shows_full_breadcrumb() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.container_presenter.set_logs(ContainerLogsDto {
        container_id: "crumb123".to_string(),
        container_name: "rc-crumb-web".to_string(),
        lines: vec![],
    });
    app.screen = Screen::ContainerLogs;

    let rendered = render_app(&mut app);
    assert!(
        rendered.contains(&format!(
            "{title}{sep}Containers{sep}rc-crumb-web logs",
            title = resources::APP_TITLE,
            sep = resources::BREADCRUMB_SEPARATOR
        )),
        "breadcrumb missing from rendered logs screen: {rendered}"
    );
}

#[test]
fn container_list_screen_shows_breadcrumb() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::ContainerList;

    let rendered = render_app(&mut app);
    assert!(
        rendered.contains(&format!(
            "{title}{sep}Containers",
            title = resources::APP_TITLE,
            sep = resources::BREADCRUMB_SEPARATOR
        )),
        "breadcrumb missing from rendered container list: {rendered}"
    );
}
