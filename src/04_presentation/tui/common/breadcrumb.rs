use super::{resources, theme::Theme};
use ratatui::{layout::Rect, widgets::Paragraph, Frame};

pub const BREADCRUMB_BAR_HEIGHT: u16 = 3;
const BREADCRUMB_HORIZONTAL_CHROME: u16 = 4;

pub fn breadcrumb_text(segments: &[&str]) -> String {
    segments.join(resources::BREADCRUMB_SEPARATOR)
}

/// Builds the breadcrumb text fitted to `max_chars` display width: the
/// current (last) segment is preserved; leading segments are dropped first,
/// and the last segment is cut from the left only when it alone overflows.
pub fn fit_breadcrumb_text(segments: &[&str], max_chars: usize) -> String {
    let full = breadcrumb_text(segments);
    if full.chars().count() <= max_chars {
        return full;
    }

    for start in 1..segments.len() {
        let tail = breadcrumb_text(&segments[start..]);
        if tail.chars().count() <= max_chars {
            return format!("{}{}", resources::TRUNCATION_MARKER, tail);
        }
    }

    let marker_len = resources::TRUNCATION_MARKER.chars().count();
    if max_chars <= marker_len {
        return resources::TRUNCATION_MARKER
            .chars()
            .take(max_chars)
            .collect();
    }
    let visible = max_chars - marker_len;
    let last = segments.last().map_or(String::new(), ToString::to_string);
    let skipped = last.chars().count().saturating_sub(visible);
    let tail: String = last.chars().skip(skipped).collect();
    format!("{}{}", resources::TRUNCATION_MARKER, tail)
}

pub fn render_breadcrumb(frame: &mut Frame, area: Rect, segments: &[&str]) {
    let max_chars = area
        .width
        .saturating_sub(BREADCRUMB_HORIZONTAL_CHROME)
        .saturating_sub(1) as usize;
    let text = fit_breadcrumb_text(segments, max_chars);

    let breadcrumb = Paragraph::new(text)
        .style(Theme::title_style())
        .block(Theme::section_block(None));
    frame.render_widget(breadcrumb, area);
}

#[cfg(test)]
mod tests {
    use super::{breadcrumb_text, fit_breadcrumb_text, render_breadcrumb, BREADCRUMB_BAR_HEIGHT};
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
    fn test_fit_breadcrumb_text_returns_full_path_when_it_fits() {
        let segments = [resources::APP_TITLE, "Containers", "web logs"];
        assert_eq!(
            fit_breadcrumb_text(&segments, 80),
            breadcrumb_text(&segments)
        );
    }

    #[test]
    fn test_fit_breadcrumb_text_drops_leading_segments_first() {
        let segments = [
            resources::APP_TITLE,
            "Containers",
            "my-extremely-long-container-name",
        ];
        assert_eq!(
            fit_breadcrumb_text(&segments, 50),
            "...Containers > my-extremely-long-container-name"
        );
    }

    #[test]
    fn test_fit_breadcrumb_text_cuts_last_segment_from_the_left() {
        let segments = [resources::APP_TITLE, "0123456789abcdefghij"];
        assert_eq!(fit_breadcrumb_text(&segments, 10), "...defghij");
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
        // The current segment stays readable, only its left side is cut.
        assert!(text.contains("exceeds-width"));
    }

    #[test]
    fn test_breadcrumb_bar_height_matches_menu_title_height() {
        assert_eq!(BREADCRUMB_BAR_HEIGHT, MAIN_MENU_TITLE_HEIGHT);
    }
}
