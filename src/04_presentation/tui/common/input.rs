use crate::presentation::tui::common::{
    action_bar::render_action_bar, resources, widgets::centered_rect, widgets::HELP_BAR_HEIGHT,
    Theme,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Clear, Paragraph, Wrap},
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

pub struct InputField<'a> {
    pub label: &'a str,
    pub value: &'a str,
    pub focused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputDialogSize {
    pub width_percent: u16,
    pub height_percent: u16,
}

impl InputDialogSize {
    pub fn new(width_percent: u16, height_percent: u16) -> Self {
        InputDialogSize {
            width_percent,
            height_percent,
        }
    }
}

impl Default for InputDialogSize {
    fn default() -> Self {
        InputDialogSize::new(60, 40)
    }
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

pub fn render_input_dialog(
    frame: &mut Frame,
    title: &str,
    fields: &[InputField<'_>],
    help_text: &str,
    size: InputDialogSize,
) {
    let area = centered_rect(size.width_percent, size.height_percent, frame.area());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(HELP_BAR_HEIGHT)])
        .split(area);
    let (dialog_area, help_area) = (chunks[0], chunks[1]);

    frame.render_widget(Clear, area);

    let mut lines: Vec<String> = Vec::new();
    for field in fields {
        lines.push(format!("{}:", field.label));
        lines.push(if field.focused {
            format!("{}{}", field.value, resources::FILTER_CURSOR)
        } else if field.value.is_empty() {
            resources::INPUT_EMPTY.to_string()
        } else {
            field.value.to_string()
        });
    }

    let dialog = Paragraph::new(lines.join("\n"))
        .block(Theme::section_block(Some(title)).border_style(Theme::selected_border_style()))
        .wrap(Wrap { trim: true });
    frame.render_widget(dialog, dialog_area);
    render_action_bar(frame, help_area, help_text);
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
    fn test_dialog_state_edits_focused_field() {
        let mut state = InputDialogState::new(2);
        state.focused_mut().push_char('x');
        assert_eq!(state.values(), vec!["x", ""]);
        state.focus_next();
        state.focused_mut().push_char('y');
        assert_eq!(state.values(), vec!["x", "y"]);
    }
}
