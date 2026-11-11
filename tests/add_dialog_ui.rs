mod common;

use common::build_app;
use crossterm::event::KeyCode;
use ratatui::{backend::TestBackend, Terminal};
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

#[tokio::test]
async fn add_container_dialog_opens_on_container_list_and_esc_closes_it() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::ContainerList;
    assert!(!app.has_modal_overlay());

    app.handle_action(AppAction::Add).await;
    assert!(app.has_modal_overlay(), "A should open the create dialog");
    let rendered = render_app(&mut app);
    assert!(
        rendered.contains("Create Container"),
        "dialog title missing: {rendered}"
    );

    assert!(app.handle_add_dialog_key(KeyCode::Char('w')));
    assert!(app.handle_add_dialog_key(KeyCode::Char('e')));
    assert!(app.handle_add_dialog_key(KeyCode::Char('b')));
    let rendered = render_app(&mut app);
    assert!(
        rendered.contains("web"),
        "typed name should render: {rendered}"
    );

    app.handle_action(AppAction::Back).await;
    assert!(!app.has_modal_overlay(), "Esc should close the dialog");
    assert_eq!(app.screen, Screen::ContainerList);
}

#[tokio::test]
async fn add_container_dialog_validation_error_keeps_dialog_open_with_popup() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::ContainerList;
    app.handle_action(AppAction::Add).await;

    // Empty fields: validation fails before any service call, so this works
    // without a Docker daemon — error popup, dialog stays open.
    app.handle_action(AppAction::Select).await;
    assert!(
        app.has_modal_overlay(),
        "dialog must stay open on validation error"
    );
    let popup = app.popup_message.as_ref().expect("expected an error popup");
    assert!(
        popup.as_str().contains("required"),
        "unexpected popup: {popup:?}"
    );
    assert!(!app.is_add_dialog_submitting());
    let rendered = render_app(&mut app);
    assert!(rendered.contains("Create Container"));

    // Dismiss the popup; the dialog is editable again, then Esc closes it.
    app.handle_action(AppAction::Select).await;
    assert!(app.popup_message.is_none());
    assert!(app.has_modal_overlay());
    app.handle_action(AppAction::Back).await;
    assert!(!app.has_modal_overlay());
}

#[tokio::test]
async fn add_container_dialog_also_opens_on_stack_containers() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::StackContainers;
    app.handle_action(AppAction::Add).await;
    assert!(app.has_modal_overlay());
    let rendered = render_app(&mut app);
    assert!(rendered.contains("Create Container"));
}

#[tokio::test]
async fn add_image_dialog_opens_on_image_list_and_esc_closes_it() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::ImageList;
    assert!(!app.has_modal_overlay());

    app.handle_action(AppAction::Add).await;
    assert!(app.has_modal_overlay(), "A should open the download dialog");
    let rendered = render_app(&mut app);
    assert!(
        rendered.contains("Download Image"),
        "dialog title missing: {rendered}"
    );

    assert!(app.handle_add_dialog_key(KeyCode::Char('n')));
    assert!(app.handle_add_dialog_key(KeyCode::Char('g')));
    let rendered = render_app(&mut app);
    assert!(
        rendered.contains("ng"),
        "typed value should render: {rendered}"
    );

    app.handle_action(AppAction::Back).await;
    assert!(!app.has_modal_overlay(), "Esc should close the dialog");
    assert_eq!(app.screen, Screen::ImageList);
}

#[tokio::test]
async fn add_image_dialog_validation_error_keeps_dialog_open_with_popup() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = Screen::ImageList;
    app.handle_action(AppAction::Add).await;

    // Empty field: validation fails before any service call, so this works
    // without a Docker daemon — error popup, dialog stays open.
    app.handle_action(AppAction::Select).await;
    assert!(
        app.has_modal_overlay(),
        "dialog must stay open on validation error"
    );
    let popup = app.popup_message.as_ref().expect("expected an error popup");
    assert!(
        popup.as_str().contains("enter an image"),
        "unexpected popup: {popup:?}"
    );
    assert!(!app.is_add_dialog_submitting());

    app.handle_action(AppAction::Select).await;
    assert!(app.popup_message.is_none());
    assert!(app.has_modal_overlay());
    app.handle_action(AppAction::Back).await;
    assert!(!app.has_modal_overlay());
}
