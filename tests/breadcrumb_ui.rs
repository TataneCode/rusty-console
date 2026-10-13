mod common;

use common::build_app;
use rusty_console::application::container::ContainerLogsDto;
use rusty_console::presentation::tui::app::{App, Screen};
use rusty_console::presentation::tui::common::resources;

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
