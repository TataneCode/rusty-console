//! Generic help popup: lists every keybinding of the current screen, one per
//! line, in a centered dialog (LazyVim-style, opened with the Space key).
//! When the command list is taller than the popup, it scrolls with j/k
//! (or ↑/↓) and shows a vertical scrollbar.

use super::{
    action_bar::{help_segment_spans, render_action_bar},
    resources,
    theme::Theme,
    widgets::{centered_rect, HELP_BAR_HEIGHT},
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::{Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

const POPUP_WIDTH_PERCENT: u16 = 60;
const POPUP_HEIGHT_PERCENT: u16 = 80;
const COMMAND_INDENT: &str = "  ";

/// Scrollable state of the help popup: tracks which command line is at the
/// top of the viewport. Content longer than the viewport scrolls with
/// [`HelpPopupState::scroll_up`] / [`HelpPopupState::scroll_down`].
#[derive(Debug, Default)]
pub struct HelpPopupState {
    pub scroll: usize,
    pub content_length: usize,
    pub viewport_length: usize,
}

impl HelpPopupState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Call once per render with the number of command lines and the number
    /// of rows available inside the popup. Keeps the scroll position valid.
    pub fn update(&mut self, content_length: usize, viewport_length: usize) {
        self.content_length = content_length;
        self.viewport_length = viewport_length;
        let max_scroll = content_length.saturating_sub(viewport_length);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        let max_scroll = self.content_length.saturating_sub(self.viewport_length);
        if self.scroll < max_scroll {
            self.scroll += 1;
        }
    }

    pub fn is_scrollable(&self) -> bool {
        self.content_length > self.viewport_length
    }
}

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

pub fn render_help_popup(frame: &mut Frame, help_text: &str, state: &mut HelpPopupState) {
    let area = centered_rect(POPUP_WIDTH_PERCENT, POPUP_HEIGHT_PERCENT, frame.area());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(HELP_BAR_HEIGHT)])
        .split(area);

    frame.render_widget(Clear, area);

    let lines = help_popup_lines(help_text);
    // Inner height of the content chunk: popup height minus top/bottom
    // borders (the block is rendered inside chunks[0]).
    let viewport_height = chunks[0].height.saturating_sub(2) as usize;
    state.update(lines.len(), viewport_height);

    let popup =
        Paragraph::new(lines).block(Theme::section_block(Some(resources::HELP_POPUP_TITLE)));
    frame.render_widget(popup.scroll((state.scroll as u16, 0)), chunks[0]);

    if state.is_scrollable() {
        let mut scrollbar_state = ScrollbarState::new(state.content_length)
            .position(state.scroll)
            .viewport_content_length(state.viewport_length);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(scrollbar, chunks[0], &mut scrollbar_state);
    }

    render_action_bar(frame, chunks[1], resources::HELP_POPUP_CLOSE_HELP);
}

#[cfg(test)]
mod tests {
    use super::{help_popup_lines, render_help_popup, HelpPopupState};
    use ratatui::{backend::TestBackend, buffer::Buffer, style::Color, Terminal};

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    #[test]
    fn test_help_popup_state_scroll_bounds() {
        let mut state = HelpPopupState::new();
        state.update(14, 10);
        assert!(state.is_scrollable());

        state.scroll_up();
        assert_eq!(state.scroll, 0, "scroll_up at top stays at 0");

        for _ in 0..10 {
            state.scroll_down();
        }
        assert_eq!(state.scroll, 4, "scroll stops at content - viewport");

        state.scroll_down();
        assert_eq!(state.scroll, 4, "scroll_down past the end stays clamped");
    }

    #[test]
    fn test_help_popup_state_not_scrollable_when_content_fits() {
        let mut state = HelpPopupState::new();
        state.update(5, 10);
        assert!(!state.is_scrollable());
        state.scroll_down();
        assert_eq!(state.scroll, 0, "no scroll when everything fits");
    }

    #[test]
    fn test_help_popup_state_update_clamps_existing_scroll() {
        let mut state = HelpPopupState::new();
        state.update(20, 5);
        for _ in 0..15 {
            state.scroll_down();
        }
        assert_eq!(state.scroll, 15);
        // Terminal grows: viewport now fits everything, scroll resets.
        state.update(20, 25);
        assert_eq!(state.scroll, 0);
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
        let mut state = HelpPopupState::new();

        terminal
            .draw(|frame| {
                render_help_popup(
                    frame,
                    " q: Quit | /: Filter | j/k: Navigate | Enter: Details | s: Start/Stop ",
                    &mut state,
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

    #[test]
    fn test_render_help_popup_scrolls_long_command_list() {
        // 14 commands on a 30-row terminal: popup is 24 rows, content area
        // holds 19 lines (24 - 2 borders - 3 close bar) — everything fits.
        let help = " c1: A | c2: B | c3: C | c4: D | c5: E | c6: F | c7: G | c8: H | c9: I | c10: J | c11: K | c12: L | c13: M | c14: N ";
        let backend = TestBackend::new(60, 14);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = HelpPopupState::new();

        // 14-row terminal: popup is 11 rows, content area holds 6 lines.
        terminal
            .draw(|frame| render_help_popup(frame, help, &mut state))
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("c1: A"));
        assert!(text.contains("c6: F"));
        assert!(
            !text.contains("c14: N"),
            "last command hidden before scroll"
        );

        for _ in 0..20 {
            state.scroll_down();
        }
        terminal
            .draw(|frame| render_help_popup(frame, help, &mut state))
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("c14: N"), "last command visible after scroll");
        assert!(
            !text.contains("c1: A"),
            "first command scrolled out of view"
        );
    }
}
