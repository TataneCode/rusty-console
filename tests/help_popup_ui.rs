mod common;

use common::build_app;
use ratatui::{backend::TestBackend, Terminal};
use rusty_console::application::network::NetworkDto;
use rusty_console::presentation::tui::app::{App, Screen};
use rusty_console::presentation::tui::common::AppAction;

fn render_app(app: &mut App) -> String {
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
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
fn list_footer_shows_only_fundamental_commands_with_space_hint() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::ContainerList;

    let rendered = render_app(&mut app);
    for hint in [
        "j/k: Navigate",
        "Enter: Details",
        "/: Filter",
        "r: Refresh",
        "q: Quit",
        "Space: All commands",
    ] {
        assert!(
            rendered.contains(hint),
            "missing footer hint {hint}: {rendered}"
        );
    }
    for full in [
        "o: Sort",
        "l: Logs",
        "e: Exec",
        "s: Start/Stop",
        "d: Delete",
    ] {
        assert!(
            !rendered.contains(full),
            "full command {full} must not stay in the footer: {rendered}"
        );
    }
}

#[tokio::test]
async fn space_opens_popup_with_full_command_list_and_escape_closes_it() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::ContainerList;

    app.handle_action(AppAction::ToggleHelp).await;
    assert!(app.help_popup_open, "Space should open the help popup");

    let rendered = render_app(&mut app);
    assert!(
        rendered.contains("Keybindings"),
        "popup title missing: {rendered}"
    );
    assert!(rendered.contains("Esc/q/Space: Close"));
    for command in [
        "j/k: Navigate",
        "Enter: Details",
        "/: Filter",
        "o: Sort",
        "l: Logs",
        "e: Exec",
        "s: Start/Stop",
        "d: Delete",
        "r: Refresh",
        "p: Pause",
        "R: Restart",
        "X: Prune",
        "A: Add",
        "q: Quit",
    ] {
        assert!(
            rendered.contains(command),
            "popup must list {command}: {rendered}"
        );
    }

    app.handle_action(AppAction::Back).await;
    assert!(!app.help_popup_open, "Esc should close the help popup");
    let rendered = render_app(&mut app);
    assert!(!rendered.contains("Keybindings"));
}

#[tokio::test]
async fn popup_is_modal_and_space_toggles_it_closed() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::VolumeList;

    app.handle_action(AppAction::ToggleHelp).await;
    assert!(app.help_popup_open);

    // Any other key is swallowed while the popup is open.
    app.handle_action(AppAction::NavigateDown).await;
    assert_eq!(app.screen, Screen::VolumeList);
    assert!(app.help_popup_open, "popup must stay open on other keys");

    app.handle_action(AppAction::ToggleHelp).await;
    assert!(!app.help_popup_open, "Space should toggle the popup closed");
}

#[tokio::test]
async fn popup_does_not_open_over_another_modal() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::ImageList;
    app.handle_action(AppAction::Add).await;
    assert!(app.has_modal_overlay(), "A should open the add dialog");

    app.handle_action(AppAction::ToggleHelp).await;
    assert!(
        !app.help_popup_open,
        "help popup must not open while the add dialog is open"
    );
}

#[tokio::test]
async fn details_screens_footer_keeps_space_hint_and_popup_lists_their_commands() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::NetworkDetails;
    app.network_presenter.set_details(NetworkDto {
        id: "net-1".to_string(),
        name: "app-net".to_string(),
        driver: "bridge".to_string(),
        scope: "local".to_string(),
        internal: true,
        attachable: false,
        subnet: Some("172.20.0.0/16".to_string()),
        gateway: Some("172.20.0.1".to_string()),
        created: "2024-01-01".to_string(),
        builtin: false,
        containers: vec![],
        can_delete: false,
        container_count: 0,
    });

    let rendered = render_app(&mut app);
    assert!(rendered.contains("Space: All commands"));
    assert!(!rendered.contains("d: Disconnect"));

    app.handle_action(AppAction::ToggleHelp).await;
    let rendered = render_app(&mut app);
    assert!(rendered.contains("Keybindings"));
    assert!(rendered.contains("d: Disconnect"));
    assert!(rendered.contains("r: Refresh"));
}
