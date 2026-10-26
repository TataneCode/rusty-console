mod common;

use common::build_app;
use ratatui::style::{Color, Modifier};

fn render_app_text_and_styles(app: &mut rusty_console::App) -> (String, Vec<(Color, Modifier)>) {
    let backend = ratatui::backend::TestBackend::new(100, 20);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();

    let buffer = terminal.backend().buffer();
    let text: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
    let styles = buffer
        .content
        .iter()
        .map(|cell| (cell.fg, cell.modifier))
        .collect();
    (text, styles)
}

#[test]
fn container_list_action_bar_shows_help_text_and_highlighted_keys() {
    let Some(mut app) = build_app() else {
        return;
    };
    app.screen = rusty_console::presentation::tui::app::Screen::ContainerList;

    let (text, styles) = render_app_text_and_styles(&mut app);
    assert!(
        text.contains("j/k: Navigate | Enter: Details | /: Filter | r: Refresh | q: Quit | Space: All commands"),
        "reduced action bar help missing: {text}"
    );
    assert!(
        !text.contains("o: Sort"),
        "full command list must move to the help popup, not the footer: {text}"
    );

    let highlighted_keys = styles
        .iter()
        .filter(|(fg, modifiers)| *fg == Color::Yellow && modifiers.contains(Modifier::BOLD))
        .count();
    assert!(
        highlighted_keys > 0,
        "action bar keys are not highlighted (yellow bold)"
    );

    let white_borders = styles.iter().filter(|(fg, _)| *fg == Color::White).count();
    assert!(
        white_borders > 0,
        "action bar/breadcrumb borders are not homogeneously white"
    );
}

#[test]
fn main_menu_action_bar_shows_help_text() {
    let Some(mut app) = build_app() else {
        return;
    };
    let (text, _) = render_app_text_and_styles(&mut app);
    assert!(text.contains("j/k: Navigate | Enter: Select | q: Quit | Space: All commands"));
}
