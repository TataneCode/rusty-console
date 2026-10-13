use super::{resources, theme::Theme, truncate_text};
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub const BREADCRUMB_BAR_HEIGHT: u16 = 3;
const BREADCRUMB_HORIZONTAL_CHROME: u16 = 4;

pub fn breadcrumb_text(segments: &[&str]) -> String {
    segments.join(resources::BREADCRUMB_SEPARATOR)
}

pub fn render_breadcrumb(frame: &mut Frame, area: Rect, segments: &[&str]) {
    let max_chars = area
        .width
        .saturating_sub(BREADCRUMB_HORIZONTAL_CHROME)
        .saturating_sub(1) as usize;
    let text = truncate_text(&breadcrumb_text(segments), max_chars);

    let breadcrumb = Paragraph::new(text).style(Theme::title_style()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::breadcrumb_border_style()),
    );
    frame.render_widget(breadcrumb, area);
}

#[cfg(test)]
mod tests {
    use super::{breadcrumb_text, render_breadcrumb, BREADCRUMB_BAR_HEIGHT};
    use crate::presentation::tui::common::resources;
    use crate::presentation::tui::common::widgets::MAIN_MENU_TITLE_HEIGHT;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    #[test]
    fn test_breadcrumb_text_joins_segments_with_separator() {
        assert_eq!(
            breadcrumb_text(&[resources::APP_TITLE, "Containers", "web logs"]),
            format!(
                "{}{}Containers{}web logs",
                resources::APP_TITLE,
                resources::BREADCRUMB_SEPARATOR,
                resources::BREADCRUMB_SEPARATOR
            )
        );
    }

    #[test]
    fn test_breadcrumb_text_single_segment_has_no_separator() {
        assert_eq!(
            breadcrumb_text(&[resources::APP_TITLE]),
            resources::APP_TITLE
        );
    }

    #[test]
    fn test_render_breadcrumb_displays_full_path() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render_breadcrumb(
                    frame,
                    frame.area(),
                    &[resources::APP_TITLE, "Containers", "web logs"],
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Rusty - docker manager > Containers > web logs"));
    }

    #[test]
    fn test_render_breadcrumb_truncates_long_paths() {
        let backend = TestBackend::new(30, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let long_name = "a-very-long-container-name-that-exceeds-width";

        terminal
            .draw(|frame| {
                render_breadcrumb(
                    frame,
                    frame.area(),
                    &[resources::APP_TITLE, "Containers", long_name],
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains(resources::TRUNCATION_MARKER));
        assert!(!text.contains(long_name));
    }

    #[test]
    fn test_breadcrumb_bar_height_matches_menu_title_height() {
        assert_eq!(BREADCRUMB_BAR_HEIGHT, MAIN_MENU_TITLE_HEIGHT);
    }
}
