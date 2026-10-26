//! Generic help popup: lists every keybinding of the current screen, one per
//! line, in a centered dialog (LazyVim-style, opened with the Space key).

use super::{
    action_bar::{help_segment_spans, render_action_bar},
    resources,
    theme::Theme,
    widgets::{centered_rect, HELP_BAR_HEIGHT},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::{Clear, Paragraph},
    Frame,
};

const POPUP_WIDTH_PERCENT: u16 = 60;
const POPUP_HEIGHT_PERCENT: u16 = 80;
const COMMAND_INDENT: &str = "  ";

/// Splits a full help string (" q: Quit | /: Filter ") into one styled line
/// per command, keys in yellow bold, labels in white.
pub fn help_popup_lines(help_text: &str) -> Vec<Line<'static>> {
    help_text
        .split('|')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut spans = vec![ratatui::text::Span::styled(
                COMMAND_INDENT,
                Theme::action_label_style(),
            )];
            spans.extend(help_segment_spans(segment));
            Line::from(spans)
        })
        .collect()
}

pub fn render_help_popup(frame: &mut Frame, help_text: &str) {
    let area = centered_rect(POPUP_WIDTH_PERCENT, POPUP_HEIGHT_PERCENT, frame.area());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(HELP_BAR_HEIGHT)])
        .split(area);

    frame.render_widget(Clear, area);

    let popup = Paragraph::new(help_popup_lines(help_text))
        .block(Theme::section_block(Some(resources::HELP_POPUP_TITLE)));
    frame.render_widget(popup, chunks[0]);
    render_action_bar(frame, chunks[1], resources::HELP_POPUP_CLOSE_HELP);
}

#[cfg(test)]
mod tests {
    use super::{help_popup_lines, render_help_popup};
    use ratatui::{backend::TestBackend, buffer::Buffer, style::Color, Terminal};

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    #[test]
    fn test_help_popup_lines_one_styled_line_per_command() {
        let lines = help_popup_lines(" q: Quit | j/k: Navigate | Enter: Details ");

        assert_eq!(lines.len(), 3);
        let rendered: String = lines[0]
            .spans
            .iter()
            .map(|s| s.content.to_string())
            .collect();
        assert_eq!(rendered, "  q: Quit");
        assert_eq!(lines[0].spans[1].style.fg, Some(Color::Yellow));
        assert_eq!(lines[0].spans[2].style.fg, Some(Color::White));
        let rendered: String = lines[2]
            .spans
            .iter()
            .map(|s| s.content.to_string())
            .collect();
        assert_eq!(rendered, "  Enter: Details");
    }

    #[test]
    fn test_help_popup_lines_keeps_full_key_names() {
        let lines = help_popup_lines(" PgUp/PgDn: Page | X: Prune ");
        let rendered: Vec<String> = lines
            .iter()
            .map(|line| line.spans.iter().map(|s| s.content.to_string()).collect())
            .collect();
        assert_eq!(rendered, vec!["  PgUp/PgDn: Page", "  X: Prune"]);
    }

    #[test]
    fn test_render_help_popup_shows_title_all_commands_and_close_hint() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                render_help_popup(
                    frame,
                    " q: Quit | /: Filter | j/k: Navigate | Enter: Details | s: Start/Stop ",
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Keybindings"));
        assert!(text.contains("q: Quit"));
        assert!(text.contains("/: Filter"));
        assert!(text.contains("j/k: Navigate"));
        assert!(text.contains("Enter: Details"));
        assert!(text.contains("s: Start/Stop"));
        assert!(text.contains("Esc/q/Space: Close"));
    }
}
