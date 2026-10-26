use super::theme::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const SEPARATOR: &str = " | ";

/// Splits one "key: Label" segment into styled spans: key in yellow bold,
/// label in white. Segments without a colon render as plain labels.
pub fn help_segment_spans(segment: &str) -> Vec<Span<'static>> {
    match segment.split_once(':') {
        Some((key, label)) => vec![
            Span::styled(key.to_string(), Theme::action_key_style()),
            Span::styled(format!(":{label}"), Theme::action_label_style()),
        ],
        None => vec![Span::styled(
            segment.to_string(),
            Theme::action_label_style(),
        )],
    }
}

/// Splits a help string (" q: Quit | /: Filter ") into styled spans:
/// keys in yellow bold, labels in white, separators dimmed.
pub fn action_bar_line(help_text: &str) -> Line<'static> {
    let mut spans = Vec::new();
    for (index, segment) in help_text.split('|').map(str::trim).enumerate() {
        if index > 0 {
            spans.push(Span::styled(SEPARATOR, Theme::help_style()));
        }
        spans.extend(help_segment_spans(segment));
    }
    Line::from(spans)
}

pub fn render_action_bar(frame: &mut Frame, area: Rect, help_text: &str) {
    let bar = Paragraph::new(action_bar_line(help_text)).block(Theme::section_block(None));
    frame.render_widget(bar, area);
}

pub fn render_loading_bar(frame: &mut Frame, area: Rect, spinner_char: &str) {
    let loading = Line::from(vec![
        Span::styled(format!(" {spinner_char} "), Theme::action_key_style()),
        Span::styled("Loading...", Theme::action_label_style()),
    ]);
    let bar = Paragraph::new(loading).block(Theme::section_block(None));
    frame.render_widget(bar, area);
}

#[cfg(test)]
mod tests {
    use super::{action_bar_line, render_action_bar, render_loading_bar};
    use ratatui::{backend::TestBackend, buffer::Buffer, style::Color, Terminal};

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    #[test]
    fn test_action_bar_line_styles_keys_and_labels() {
        let line = action_bar_line(" q: Quit | /: Filter ");

        assert_eq!(line.spans.len(), 5);
        assert_eq!(line.spans[0].content, "q");
        assert_eq!(line.spans[0].style.fg, Some(Color::Yellow));
        assert_eq!(line.spans[1].content, ": Quit");
        assert_eq!(line.spans[1].style.fg, Some(Color::White));
        assert_eq!(line.spans[2].content, " | ");
        assert_eq!(line.spans[2].style.fg, Some(Color::DarkGray));
        assert_eq!(line.spans[3].content, "/");
        assert_eq!(line.spans[3].style.fg, Some(Color::Yellow));
        assert_eq!(line.spans[4].content, ": Filter");
    }

    #[test]
    fn test_action_bar_line_reproduces_trimmed_text() {
        let line = action_bar_line(" q: Quit | j/k: Navigate | d: Delete ");
        let rendered: String = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert_eq!(rendered, "q: Quit | j/k: Navigate | d: Delete");
    }

    #[test]
    fn test_action_bar_line_segment_without_colon_is_label() {
        let line = action_bar_line(" plain info ");
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "plain info");
        assert_eq!(line.spans[0].style.fg, Some(Color::White));
    }

    #[test]
    fn test_render_action_bar_displays_help_text() {
        let backend = TestBackend::new(80, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| render_action_bar(frame, frame.area(), " q: Quit | j/k: Navigate "))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("q: Quit | j/k: Navigate"));
    }

    #[test]
    fn test_render_loading_bar_displays_spinner_and_loading_text() {
        let backend = TestBackend::new(80, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| render_loading_bar(frame, frame.area(), "|"))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("| Loading..."));
    }
}
