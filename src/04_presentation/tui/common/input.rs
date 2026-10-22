use crate::presentation::tui::common::{action_bar::render_action_bar, resources, Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Clear, Padding, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInputState {
    value: String,
}

impl TextInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn push_char(&mut self, c: char) {
        self.value.push(c);
    }

    pub fn pop_char(&mut self) {
        self.value.pop();
    }

    pub fn clear(&mut self) {
        self.value.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

pub struct InputField<'text> {
    pub label: &'text str,
    pub value: &'text str,
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputDialogState {
    fields: Vec<TextInputState>,
    focused: usize,
}

impl InputDialogState {
    pub fn new(field_count: usize) -> Self {
        InputDialogState {
            fields: (0..field_count).map(|_| TextInputState::new()).collect(),
            focused: 0,
        }
    }

    pub fn focused_index(&self) -> usize {
        self.focused
    }

    pub fn focus_next(&mut self) {
        if self.focused + 1 < self.fields.len() {
            self.focused += 1;
        }
    }

    pub fn focus_previous(&mut self) {
        if self.focused > 0 {
            self.focused -= 1;
        }
    }

    pub fn focused_mut(&mut self) -> &mut TextInputState {
        &mut self.fields[self.focused]
    }

    pub fn values(&self) -> Vec<&str> {
        self.fields.iter().map(|f| f.value()).collect()
    }
}

const LABEL_HEIGHT: u16 = 1;
const TEXT_BOX_HEIGHT: u16 = 3;
const FIELD_SPACING: u16 = 1;
const VERTICAL_PADDING: u16 = 2;
const HELP_BAR_HEIGHT: u16 = 3;

/// Height of an input dialog wrapping the given number of fields,
/// including the help bar and the dialog borders.
pub fn input_dialog_height(field_count: usize) -> u16 {
    field_count as u16 * (LABEL_HEIGHT + TEXT_BOX_HEIGHT + FIELD_SPACING) - FIELD_SPACING
        + HELP_BAR_HEIGHT
        + VERTICAL_PADDING
        + 1
}

pub fn render_input_dialog(
    frame: &mut Frame,
    title: &str,
    fields: &[InputField<'_>],
    help_text: &str,
    width_percent: u16,
) {
    let dialog_height = input_dialog_height(fields.len());
    let area = vertically_centered_rect(width_percent, dialog_height, frame.area());
    frame.render_widget(Clear, area);

    let dialog = Paragraph::new("").block(Theme::section_block(Some(title)));
    frame.render_widget(dialog, area);
    let inner = Theme::section_block(None).inner(area);
    let inner_area = Rect {
        y: inner.y + 1,
        height: inner.height.saturating_sub(HELP_BAR_HEIGHT + 1),
        ..inner
    };

    let mut constraints: Vec<Constraint> = Vec::new();
    for index in 0..fields.len() {
        if index > 0 {
            constraints.push(Constraint::Length(FIELD_SPACING));
        }
        constraints.push(Constraint::Length(LABEL_HEIGHT));
        constraints.push(Constraint::Length(TEXT_BOX_HEIGHT));
    }
    let field_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    let mut area_index = 0;
    for (position, field) in fields.iter().enumerate() {
        if position > 0 {
            area_index += 1;
        }
        let label_area = field_areas[area_index];
        let box_area = field_areas[area_index + 1];
        area_index += 2;

        let label = Paragraph::new(format!("{}:", field.label)).style(Theme::action_label_style());
        frame.render_widget(label, label_area);

        let border_style = if field.focused {
            Theme::selected_border_style()
        } else {
            Theme::border_style()
        };
        let input_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .padding(Padding::horizontal(1));
        let text = if field.focused {
            format!("{}{}", field.value, resources::FILTER_CURSOR)
        } else if field.value.is_empty() {
            resources::INPUT_EMPTY.to_string()
        } else {
            field.value.to_string()
        };
        let input = Paragraph::new(text).block(input_block);
        frame.render_widget(input, box_area);
    }

    let help_area = Rect {
        y: area.bottom().saturating_sub(HELP_BAR_HEIGHT),
        height: HELP_BAR_HEIGHT,
        ..area
    };
    render_action_bar(frame, help_area, help_text);
}

pub(crate) fn vertically_centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_push_and_pop() {
        let mut input = TextInputState::new();
        input.push_char('a');
        input.push_char('b');
        assert_eq!(input.value(), "ab");
        input.pop_char();
        assert_eq!(input.value(), "a");
        assert!(!input.is_empty());
        input.clear();
        assert!(input.is_empty());
    }

    #[test]
    fn test_dialog_state_focus_navigation() {
        let mut state = InputDialogState::new(3);
        assert_eq!(state.focused_index(), 0);
        state.focus_next();
        state.focus_next();
        assert_eq!(state.focused_index(), 2);
        state.focus_next();
        assert_eq!(state.focused_index(), 2);
        state.focus_previous();
        assert_eq!(state.focused_index(), 1);
    }

    #[test]
    fn test_input_dialog_height_scales_with_field_count() {
        let one = input_dialog_height(1);
        let three = input_dialog_height(3);
        assert!(one < three);
        assert!(three <= 20);
    }

    #[test]
    fn test_render_input_dialog_draws_bordered_textboxes_with_focus() {
        use ratatui::{backend::TestBackend, buffer::Buffer, style::Color, Terminal};
        fn buffer_text(buffer: &Buffer) -> String {
            buffer
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
        }
        let backend = TestBackend::new(60, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let fields = vec![
                    InputField {
                        label: "Name",
                        value: "web",
                        focused: true,
                    },
                    InputField {
                        label: "Image",
                        value: "",
                        focused: false,
                    },
                ];
                render_input_dialog(frame, " Create ", &fields, " Enter: OK | Esc: Cancel ", 60);
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let text = buffer_text(buffer);
        assert!(text.contains("Name"));
        assert!(text.contains("web"));
        assert!(text.contains('\u{256d}'));
        let has_focused_cyan = buffer.content.iter().any(|cell| cell.fg == Color::Cyan);
        assert!(has_focused_cyan);
        let cursor_visible = text.contains('\u{258f}');
        assert!(cursor_visible);
    }

    #[test]
    fn test_dialog_state_edits_focused_field() {
        let mut state = InputDialogState::new(2);
        state.focused_mut().push_char('x');
        assert_eq!(state.values(), vec!["x", ""]);
        state.focus_next();
        state.focused_mut().push_char('y');
        assert_eq!(state.values(), vec!["x", "y"]);
    }
}
