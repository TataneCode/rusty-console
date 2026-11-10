mod common;

use common::build_app;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rusty_console::application::container::ContainerDto;
use rusty_console::application::image::ImageDto;
use rusty_console::application::network::{NetworkContainerDto, NetworkDto};
use rusty_console::application::volume::VolumeDto;
use rusty_console::presentation::tui::app::Screen;
use rusty_console::presentation::tui::common::keys::{
    map_key_to_action, map_key_to_action_for_context, AppAction, KeyContext,
};
use rusty_console::{App, ContainerState};

fn render_app_text(app: &mut App) -> String {
    let backend = ratatui::backend::TestBackend::new(120, 24);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

fn make_container() -> ContainerDto {
    ContainerDto {
        id: "abc123".to_string(),
        short_id: "abc123".to_string(),
        name: "web".to_string(),
        image: "nginx:latest".to_string(),
        state: ContainerState::Running,
        status: "Up 5 minutes".to_string(),
        created: "2024-01-01".to_string(),
        ports: "80:80".to_string(),
        networks: "bridge".to_string(),
        can_start: false,
        can_stop: true,
        can_delete: true,
        can_restart: true,
        can_pause: true,
        can_unpause: false,
        env_vars: vec!["RUST_LOG=info".to_string()],
        runtime_stats: None,
    }
}

fn make_volume() -> VolumeDto {
    VolumeDto {
        id: "vol-1".to_string(),
        name: "db-data".to_string(),
        driver: "local".to_string(),
        mountpoint: "/var/lib/docker/volumes/db-data/_data".to_string(),
        size: "10 MB".to_string(),
        created: "2024-01-01".to_string(),
        in_use: true,
        linked_containers: vec!["/web".to_string()],
        can_delete: false,
    }
}

fn make_image() -> ImageDto {
    ImageDto {
        id: "sha256:abc".to_string(),
        short_id: "abc".to_string(),
        repository: "nginx".to_string(),
        tag: "latest".to_string(),
        full_name: "nginx:latest".to_string(),
        size: "12 MB".to_string(),
        created: "2024-01-01".to_string(),
        in_use: true,
        is_dangling: false,
        can_delete: true,
    }
}

fn make_network() -> NetworkDto {
    NetworkDto {
        id: "net-1".to_string(),
        name: "app-net".to_string(),
        driver: "bridge".to_string(),
        scope: "local".to_string(),
        internal: false,
        attachable: false,
        subnet: Some("172.20.0.0/16".to_string()),
        gateway: Some("172.20.0.1".to_string()),
        created: "2024-01-01".to_string(),
        builtin: false,
        containers: vec![NetworkContainerDto {
            name: "web".to_string(),
            ip: "172.20.0.2/16".to_string(),
        }],
        can_delete: false,
        container_count: 1,
    }
}

#[test]
fn details_help_bars_show_y_copy() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::ContainerDetails;
    app.container_presenter.set_details(make_container());
    let text = render_app_text(&mut app);
    assert!(
        text.contains("y: Copy"),
        "container details help bar should show y: Copy: {text}"
    );

    app.screen = Screen::VolumeDetails;
    app.volume_presenter.set_details(make_volume());
    let text = render_app_text(&mut app);
    assert!(
        text.contains("y: Copy"),
        "volume details help bar should show y: Copy: {text}"
    );

    app.screen = Screen::ImageDetails;
    app.image_presenter.set_details(make_image());
    let text = render_app_text(&mut app);
    assert!(
        text.contains("y: Copy"),
        "image details help bar should show y: Copy: {text}"
    );

    app.screen = Screen::NetworkDetails;
    app.network_presenter.set_details(make_network());
    let text = render_app_text(&mut app);
    assert!(
        text.contains("y: Copy"),
        "network details help bar should show y: Copy: {text}"
    );
}

#[test]
fn y_key_maps_to_copy_on_details_contexts_only() {
    let y = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
    assert_eq!(
        map_key_to_action_for_context(y, KeyContext::Details),
        Some(AppAction::Copy)
    );
    assert_eq!(
        map_key_to_action_for_context(y, KeyContext::NetworkDetails),
        Some(AppAction::Copy)
    );
    assert_eq!(
        map_key_to_action_for_context(y, KeyContext::ContainerLogs),
        Some(AppAction::CopyLogLine)
    );
    assert_eq!(map_key_to_action_for_context(y, KeyContext::Global), None);
}

#[test]
fn ctrl_shift_c_x_v_map_to_clipboard_actions_and_ctrl_c_still_quits() {
    let ctrl_shift = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    assert_eq!(
        map_key_to_action(KeyEvent::new(KeyCode::Char('c'), ctrl_shift)),
        Some(AppAction::Copy)
    );
    assert_eq!(
        map_key_to_action(KeyEvent::new(KeyCode::Char('x'), ctrl_shift)),
        Some(AppAction::Cut)
    );
    assert_eq!(
        map_key_to_action(KeyEvent::new(KeyCode::Char('v'), ctrl_shift)),
        Some(AppAction::Paste)
    );
    assert_eq!(
        map_key_to_action(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Some(AppAction::Quit)
    );
    let copy_key = KeyEvent::new(KeyCode::Char('c'), ctrl_shift);
    assert_eq!(
        map_key_to_action_for_context(copy_key, KeyContext::Global),
        None
    );
    assert_eq!(
        map_key_to_action_for_context(copy_key, KeyContext::Details),
        Some(AppAction::Copy)
    );
}

#[tokio::test]
async fn copy_action_on_container_details_copies_only_the_selection() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::ContainerDetails;
    app.container_presenter.set_details(make_container());
    // No selection: copy is a no-op
    app.handle_action(AppAction::Copy).await;
    assert!(
        app.popup_message.is_none(),
        "copy without a text selection must be a no-op"
    );
    // With a selection: the selected text is copied (popup feedback)
    app.details_selection_text = Some("abc123".to_string());
    app.handle_action(AppAction::Copy).await;
    assert!(
        app.popup_message.is_some(),
        "copy action should show a feedback popup (success or clipboard error)"
    );
}

#[test]
fn render_extracts_mouse_selection_text_on_details_screen() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::ContainerDetails;
    app.container_presenter.set_details(make_container());
    // Select the first two text lines of the details block
    // (120x24 terminal: breadcrumb 3 rows, help bar 3 rows, border 1 row).
    app.details_selection = Some(((2, 4), (40, 5)));
    let _ = render_app_text(&mut app);
    let text = app
        .details_selection_text
        .clone()
        .expect("selection text should be extracted on render");
    assert!(text.contains("abc123"), "selected text: {text:?}");
    assert!(text.contains("web"), "selected text: {text:?}");
    assert_eq!(text.lines().count(), 2, "selected text: {text:?}");
}

#[tokio::test]
async fn copy_action_on_network_details_falls_back_to_selected_container() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::NetworkDetails;
    app.network_presenter.set_details(make_network());
    // No mouse selection: falls back to the j/k-selected container line
    app.handle_action(AppAction::Copy).await;
    assert!(
        app.popup_message.is_some(),
        "copy action should show a feedback popup (success or clipboard error)"
    );
}

#[test]
fn clipboard_shortcut_consumed_in_filter_but_not_for_plain_letters() {
    let Some(mut app) = build_app() else {
        return;
    };
    let ctrl_shift = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    app.screen = Screen::ContainerList;
    app.container_presenter.activate_filter();
    app.container_presenter.push_filter_char('a');
    assert!(
        !app.handle_input_clipboard_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty()))
    );
    assert!(app.handle_input_clipboard_key(KeyEvent::new(KeyCode::Char('c'), ctrl_shift)));
    // Dismiss a possible clipboard-error popup (headless): while a popup is
    // open, clipboard shortcuts are not consumed (dismissed by next key first)
    app.popup_message = None;
    assert!(app.handle_input_clipboard_key(KeyEvent::new(KeyCode::Char('x'), ctrl_shift)));
    assert!(
        app.container_presenter.filter.value().is_empty(),
        "cut should clear the filter value"
    );
    app.popup_message = None;
    assert!(app.handle_input_clipboard_key(KeyEvent::new(KeyCode::Char('v'), ctrl_shift)));
}

#[tokio::test]
async fn clipboard_shortcut_cut_clears_add_dialog_field() {
    let Some(mut app) = build_app() else {
        return;
    };
    let ctrl_shift = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    app.screen = Screen::ImageList;
    app.handle_action(AppAction::Add).await;
    assert!(app.handle_add_dialog_key(KeyCode::Char('z')));
    assert!(app.handle_add_dialog_key(KeyCode::Char('z')));
    let text = render_app_text(&mut app);
    assert!(text.contains("zz"), "typed value should be visible: {text}");
    assert!(app.handle_input_clipboard_key(KeyEvent::new(KeyCode::Char('x'), ctrl_shift)));
    let text = render_app_text(&mut app);
    assert!(
        !text.contains("zz"),
        "cut should clear the focused field: {text}"
    );
}
