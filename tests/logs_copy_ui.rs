mod common;

use common::build_app;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Modifier};
use rusty_console::application::container::{ContainerLogsDto, LogLineDto};
use rusty_console::presentation::tui::app::Screen;
use rusty_console::presentation::tui::common::keys::{
    map_key_to_action_for_context, AppAction, KeyContext,
};

fn logs_app() -> Option<rusty_console::App> {
    let mut app = build_app()?;
    app.container_presenter.set_logs(ContainerLogsDto {
        container_id: "copy123".to_string(),
        container_name: "rc-copy-web".to_string(),
        lines: vec![
            LogLineDto {
                timestamp: None,
                message: "first line".to_string(),
            },
            LogLineDto {
                timestamp: None,
                message: "second line".to_string(),
            },
        ],
    });
    app.screen = Screen::ContainerLogs;
    Some(app)
}

fn render_app_text_and_styles(
    app: &mut rusty_console::App,
) -> (String, Vec<(Color, Color, Modifier)>) {
    let backend = ratatui::backend::TestBackend::new(100, 20);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();

    let buffer = terminal.backend().buffer();
    let text: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
    let styles = buffer
        .content
        .iter()
        .map(|cell| (cell.bg, cell.fg, cell.modifier))
        .collect();
    (text, styles)
}

fn find_text_cell_index(buffer: &ratatui::buffer::Buffer, target: &str) -> Option<usize> {
    let symbols: Vec<&str> = buffer.content.iter().map(|cell| cell.symbol()).collect();
    (0..symbols.len()).find(|&i| {
        target
            .chars()
            .enumerate()
            .all(|(k, ch)| symbols.get(i + k).and_then(|s| s.chars().next()) == Some(ch))
    })
}

#[test]
fn logs_screen_help_bar_shows_copy_action() {
    let Some(mut app) = logs_app() else {
        return;
    };
    let (text, _) = render_app_text_and_styles(&mut app);
    assert!(
        text.contains("y: Copy"),
        "logs help bar should show y: Copy: {text}"
    );
}

#[test]
fn logs_screen_selected_line_is_highlighted() {
    let Some(mut app) = logs_app() else {
        return;
    };
    let backend = ratatui::backend::TestBackend::new(100, 20);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let buffer = terminal.backend().buffer();

    let selected = find_text_cell_index(buffer, "first line").expect("selected line present");
    let other = find_text_cell_index(buffer, "second line").expect("other line present");

    let cell = &buffer.content[selected];
    assert!(
        cell.modifier.contains(Modifier::BOLD),
        "selected log line should be bold"
    );
    assert_eq!(
        cell.bg,
        Color::DarkGray,
        "selected log line should have the highlight background"
    );
    assert_eq!(
        cell.fg,
        Color::Reset,
        "selected log line keeps the default foreground (highlight sets no fg)"
    );

    let cell = &buffer.content[other];
    assert!(
        !cell.modifier.contains(Modifier::BOLD),
        "non-selected log line should not be bold"
    );
    assert_eq!(
        cell.bg,
        Color::Reset,
        "non-selected log line should have no highlight background"
    );
}

#[tokio::test]
async fn copy_log_line_action_shows_feedback_popup() {
    let Some(mut app) = logs_app() else {
        return;
    };
    assert!(app.popup_message.is_none());
    app.handle_action(AppAction::CopyLogLine).await;
    assert!(
        app.popup_message.is_some(),
        "copy action should show a feedback popup (success or clipboard error)"
    );
}

#[test]
fn y_key_maps_to_copy_only_on_logs_screen() {
    let key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty());
    assert_eq!(
        map_key_to_action_for_context(key, KeyContext::ContainerLogs),
        Some(AppAction::CopyLogLine)
    );
    assert_eq!(map_key_to_action_for_context(key, KeyContext::Global), None);
}
