mod common;

use common::build_app;
use crossterm::event::KeyCode;
use ratatui::{backend::TestBackend, Terminal};
use rusty_console::application::stack::{StackContainerDto, StackDto};
use rusty_console::domain::stack::StackContainerState;
use rusty_console::presentation::tui::app::{App, Screen};
use rusty_console::presentation::tui::common::{resources, AppAction};

fn render_app(app: &mut App) -> String {
    let backend = TestBackend::new(140, 24);
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

fn make_stack_container(name: &str, image: &str, running: bool) -> StackContainerDto {
    StackContainerDto {
        id: format!("id-{name}"),
        name: name.to_string(),
        image: image.to_string(),
        state: if running {
            StackContainerState::Running
        } else {
            StackContainerState::Exited
        },
        status: if running {
            "Up 5 minutes".to_string()
        } else {
            "Exited (0) 2 minutes ago".to_string()
        },
        ports: "80:80".to_string(),
        can_start: !running,
        can_stop: running,
        can_restart: running,
        can_pause: running,
        can_unpause: false,
        runtime_stats: None,
    }
}

fn app_on_stack_containers() -> Option<App> {
    let mut app = build_app()?;
    app.stack_presenter.set_stacks(vec![StackDto {
        name: "compose-app".to_string(),
        container_count: 2,
        running_count: 1,
        containers: vec![
            make_stack_container("web", "nginx:latest", true),
            make_stack_container("db", "postgres:16", false),
        ],
    }]);
    app.screen = Screen::StackList;
    Some(app)
}

fn drill_into_stack(app: &mut App) {
    app.stack_presenter.set_stack_containers(vec![
        make_stack_container("web", "nginx:latest", true),
        make_stack_container("db", "postgres:16", false),
    ]);
    app.screen = Screen::StackContainers;
}

#[test]
fn stack_containers_screen_matches_container_list_layout_and_help() {
    let Some(mut app) = app_on_stack_containers() else {
        return;
    };
    drill_into_stack(&mut app);

    let rendered = render_app(&mut app);
    for header in resources::CONTAINER_HEADERS {
        assert!(
            rendered.contains(header),
            "missing column header {header} on stack containers screen: {rendered}"
        );
    }
    assert!(rendered.contains("Stack: compose-app"));
    assert!(rendered.contains("web"));
    assert!(rendered.contains("db"));
    // Same per-container actions as the container list, minus stack-level ones.
    for hint in [
        "l: Logs",
        "e: Exec",
        "s: Start/Stop",
        "d: Delete",
        "Enter: Details",
        "p: Pause",
        "R: Restart",
        "X: Prune",
        "A: Add",
        "o: Sort",
    ] {
        assert!(
            rendered.contains(hint),
            "missing help hint {hint} on stack containers screen: {rendered}"
        );
    }
    assert!(
        !rendered.contains("Stop All"),
        "stack-level Stop All must live on the stack list, not on stack containers: {rendered}"
    );
    assert!(
        !rendered.contains("Remove All"),
        "stack-level Remove All must live on the stack list, not on stack containers: {rendered}"
    );
}

#[test]
fn stack_list_screen_owns_stack_level_actions() {
    let Some(mut app) = app_on_stack_containers() else {
        return;
    };

    let rendered = render_app(&mut app);
    assert!(rendered.contains("s: Start All"));
    assert!(rendered.contains("S: Stop All"));
    assert!(rendered.contains("D: Remove All"));
}

#[tokio::test]
async fn stack_containers_filter_shows_contextual_help_and_narrows_rows() {
    let Some(mut app) = app_on_stack_containers() else {
        return;
    };
    drill_into_stack(&mut app);

    app.handle_action(AppAction::ActivateFilter).await;
    for c in "db".chars() {
        assert!(app.handle_filter_key(KeyCode::Char(c)));
    }

    let rendered = render_app(&mut app);
    assert!(
        rendered.contains("[/: db"),
        "filter prompt missing from title: {rendered}"
    );
    assert!(rendered.contains("Type: Filter"));
    assert!(rendered.contains("Enter: Details"));
    assert!(rendered.contains("Esc: Clear"));
    assert!(
        !rendered.contains("Start/Stop"),
        "letter-key actions must be hidden while filtering: {rendered}"
    );
    assert!(
        app.stack_presenter
            .filtered_stack_containers()
            .iter()
            .all(|c| c.name == "db"),
        "filter should narrow the stack container list to db"
    );

    assert!(app.handle_filter_key(KeyCode::Esc));
    assert_eq!(
        app.stack_presenter.filtered_stack_containers().len(),
        2,
        "Esc should clear the filter"
    );
}

#[tokio::test]
async fn stack_containers_sort_toggle_reorders_rows() {
    let Some(mut app) = app_on_stack_containers() else {
        return;
    };
    drill_into_stack(&mut app);

    app.handle_action(AppAction::ToggleSort).await;
    let names: Vec<&str> = app
        .stack_presenter
        .filtered_stack_containers()
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(names, vec!["db", "web"], "first toggle should sort A→Z");

    app.handle_action(AppAction::ToggleSort).await;
    let names: Vec<&str> = app
        .stack_presenter
        .filtered_stack_containers()
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(names, vec!["web", "db"], "second toggle should sort Z→A");
}

#[tokio::test]
async fn stack_list_remove_all_opens_confirmation_and_executes_on_stack() {
    let Some(mut app) = app_on_stack_containers() else {
        return;
    };

    app.handle_action(AppAction::RemoveAll).await;
    assert!(
        app.confirm_dialog.is_some(),
        "D on the stack list should open the remove-all confirmation"
    );
}

#[tokio::test]
async fn stack_containers_has_no_stack_level_action_handlers() {
    let Some(mut app) = app_on_stack_containers() else {
        return;
    };
    drill_into_stack(&mut app);

    // Stop All / Start All / Remove All are stack-list actions: on the
    // containers screen they must be inert (no dialog, no screen change).
    for action in [
        AppAction::StopAll,
        AppAction::StartAll,
        AppAction::RemoveAll,
    ] {
        app.handle_action(action).await;
        assert!(
            app.confirm_dialog.is_none(),
            "{action:?} must not open a dialog on the stack containers screen"
        );
        assert_eq!(app.screen, Screen::StackContainers);
    }
}

#[tokio::test]
async fn stack_containers_loading_replaces_help_bar() {
    let Some(mut app) = app_on_stack_containers() else {
        return;
    };
    drill_into_stack(&mut app);
    app.stack_presenter.containers.set_loading(true);

    let rendered = render_app(&mut app);
    assert!(
        rendered.contains("Loading..."),
        "loading spinner missing on stack containers screen: {rendered}"
    );
    assert!(
        !rendered.contains("Start/Stop"),
        "help bar must be replaced while loading: {rendered}"
    );
}
