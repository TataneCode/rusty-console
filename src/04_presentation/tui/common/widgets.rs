use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use super::{resources, theme::Theme};

#[derive(Debug, Clone)]
pub enum PopupMessage {
    Error(String),
    Info(String),
}

impl PopupMessage {
    pub fn as_str(&self) -> &str {
        match self {
            PopupMessage::Error(msg) | PopupMessage::Info(msg) => msg,
        }
    }
}

pub const HELP_BAR_HEIGHT: u16 = 2;
pub const MAIN_MENU_TITLE_HEIGHT: u16 = 3;
pub const CONFIRM_DIALOG_WIDTH_PERCENT: u16 = 50;
pub const CONFIRM_DIALOG_HEIGHT_PERCENT: u16 = 20;
pub const ERROR_DIALOG_WIDTH_PERCENT: u16 = 60;
pub const ERROR_DIALOG_HEIGHT_PERCENT: u16 = 20;
pub const SELECTION_DIALOG_WIDTH_PERCENT: u16 = 40;
pub const SELECTION_DIALOG_HEIGHT_PERCENT: u16 = 30;

pub struct TableSelection {
    pub state: TableState,
    pub items_count: usize,
}

impl TableSelection {
    pub fn new() -> Self {
        TableSelection {
            state: TableState::default(),
            items_count: 0,
        }
    }

    pub fn set_items(&mut self, count: usize) {
        self.items_count = count;
        if count > 0 && self.state.selected().is_none() {
            self.state.select(Some(0));
        } else if count == 0 {
            self.state.select(None);
        } else if let Some(selected) = self.state.selected() {
            if selected >= count {
                self.state.select(Some(count.saturating_sub(1)));
            }
        }
    }

    pub fn next(&mut self) {
        if self.items_count == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items_count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items_count == 0 {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items_count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn select(&mut self, index: Option<usize>) {
        match index {
            Some(index) if index < self.items_count => self.state.select(Some(index)),
            Some(_) if self.items_count > 0 => self.state.select(Some(self.items_count - 1)),
            _ => self.state.select(None),
        }
    }
}

impl Default for TableSelection {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_table<'a>(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    headers: Vec<&'a str>,
    rows: Vec<Row<'a>>,
    widths: Vec<Constraint>,
    state: &mut TableState,
) {
    let header_cells: Vec<_> = headers
        .into_iter()
        .map(|h| ratatui::widgets::Cell::from(h).style(Theme::header_style()))
        .collect();

    let header = Row::new(header_cells).height(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(Theme::title_style())
                .border_style(Theme::border_style()),
        )
        .row_highlight_style(Theme::highlight_style());

    frame.render_stateful_widget(table, area, state);
}

pub fn render_help(frame: &mut Frame, area: Rect, help_text: &str) {
    let help = Paragraph::new(help_text)
        .style(Theme::help_style())
        .block(Block::default().borders(Borders::TOP));

    frame.render_widget(help, area);
}

pub fn render_confirm_dialog(frame: &mut Frame, message: &str, selected_yes: bool) {
    let area = centered_rect(
        CONFIRM_DIALOG_WIDTH_PERCENT,
        CONFIRM_DIALOG_HEIGHT_PERCENT,
        frame.area(),
    );

    frame.render_widget(Clear, area);

    let yes_style = if selected_yes {
        Theme::highlight_style()
    } else {
        Theme::default_style()
    };

    let no_style = if !selected_yes {
        Theme::highlight_style()
    } else {
        Theme::default_style()
    };

    let text = format!(
        "{}\n\n{}  {}",
        message,
        resources::CONFIRM_YES_LABEL,
        resources::CONFIRM_NO_LABEL
    );

    let dialog = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(resources::CONFIRM_TITLE)
                .title_style(Theme::title_style())
                .border_style(Theme::selected_border_style()),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(dialog, area);

    // Render buttons with proper styling
    let button_area = Rect {
        x: area.x + 2,
        y: area.y + 3,
        width: area.width - 4,
        height: 1,
    };

    let yes_text = Paragraph::new(resources::CONFIRM_YES_LABEL).style(yes_style);
    let no_text = Paragraph::new(resources::CONFIRM_NO_LABEL).style(no_style);

    let yes_area = Rect {
        x: button_area.x,
        y: button_area.y,
        width: resources::CONFIRM_YES_BUTTON_WIDTH,
        height: 1,
    };

    let no_area = Rect {
        x: button_area.x + resources::CONFIRM_BUTTON_SPACING,
        y: button_area.y,
        width: resources::CONFIRM_NO_BUTTON_WIDTH,
        height: 1,
    };

    frame.render_widget(yes_text, yes_area);
    frame.render_widget(no_text, no_area);
}

pub fn render_popup_message(frame: &mut Frame, message: &PopupMessage) {
    let area = centered_rect(
        ERROR_DIALOG_WIDTH_PERCENT,
        ERROR_DIALOG_HEIGHT_PERCENT,
        frame.area(),
    );

    frame.render_widget(Clear, area);

    let (style, title) = match message {
        PopupMessage::Error(_) => (Theme::error_style(), resources::ERROR_TITLE),
        PopupMessage::Info(_) => (Theme::info_style(), resources::INFO_TITLE),
    };

    let popup_widget = Paragraph::new(message.as_str())
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(style)
                .border_style(style),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(popup_widget, area);
}

pub fn render_selection_dialog(
    frame: &mut Frame,
    title: &str,
    options: &[&str],
    state: &mut ListState,
    help_text: &str,
) {
    let area = centered_rect(
        SELECTION_DIALOG_WIDTH_PERCENT,
        SELECTION_DIALOG_HEIGHT_PERCENT,
        frame.area(),
    );
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(HELP_BAR_HEIGHT)])
        .split(area);

    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = options.iter().copied().map(ListItem::new).collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(Theme::title_style())
                .border_style(Theme::selected_border_style()),
        )
        .highlight_style(Theme::highlight_style())
        .highlight_symbol(resources::MAIN_MENU_HIGHLIGHT_SYMBOL);
    let help = Paragraph::new(help_text)
        .style(Theme::help_style())
        .block(Block::default().borders(Borders::TOP));

    frame.render_stateful_widget(list, chunks[0], state);
    frame.render_widget(help, chunks[1]);
}

pub fn split_content_area(area: Rect) -> [Rect; 2] {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(HELP_BAR_HEIGHT)])
        .split(area);

    [chunks[0], chunks[1]]
}

pub fn split_menu_area(area: Rect) -> [Rect; 3] {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(MAIN_MENU_TITLE_HEIGHT),
            Constraint::Min(0),
            Constraint::Length(HELP_BAR_HEIGHT),
        ])
        .split(area);

    [chunks[0], chunks[1], chunks[2]]
}

pub fn truncate_text(text: &str, max_chars: usize) -> String {
    let marker_len = resources::TRUNCATION_MARKER.chars().count();
    if max_chars <= marker_len {
        return resources::TRUNCATION_MARKER
            .chars()
            .take(max_chars)
            .collect();
    }

    let visible_chars = max_chars - marker_len;
    let mut visible_end = text.len();
    let mut chars_seen = 0;

    for (idx, _) in text.char_indices() {
        if chars_seen == visible_chars {
            visible_end = idx;
        }

        chars_seen += 1;
        if chars_seen > max_chars {
            return format!("{}{}", &text[..visible_end], resources::TRUNCATION_MARKER);
        }
    }

    text.to_string()
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_width = r.width * percent_x / 100;
    let popup_height = r.height * percent_y / 100;
    let popup_x = (r.width - popup_width) / 2;
    let popup_y = (r.height - popup_height) / 2;

    Rect {
        x: r.x + popup_x,
        y: r.y + popup_y,
        width: popup_width,
        height: popup_height,
    }
}

#[cfg(test)]
mod tests {
    use super::{render_selection_dialog, truncate_text, PopupMessage, TableSelection};
    use ratatui::{backend::TestBackend, buffer::Buffer, widgets::ListState, Terminal};

    #[test]
    fn test_truncate_text_handles_utf8_boundaries() {
        assert_eq!(truncate_text("ééééé", 4), "é...");
        assert_eq!(truncate_text("🦀🦀🦀🦀🦀", 4), "🦀...");
    }

    #[test]
    fn test_table_selection_select_clamps_to_last_item() {
        let mut selection = TableSelection::new();
        selection.set_items(2);

        selection.select(Some(99));

        assert_eq!(selection.selected(), Some(1));
    }

    #[test]
    fn test_popup_message_as_str_returns_inner_text() {
        let error = PopupMessage::Error("oops".to_string());
        assert_eq!(error.as_str(), "oops");

        let info = PopupMessage::Info("done".to_string());
        assert_eq!(info.as_str(), "done");
    }

    #[test]
    fn test_popup_message_variants_are_distinct() {
        let error = PopupMessage::Error("msg".to_string());
        let info = PopupMessage::Info("msg".to_string());
        assert!(matches!(error, PopupMessage::Error(_)));
        assert!(matches!(info, PopupMessage::Info(_)));
    }

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    #[test]
    fn test_render_selection_dialog_shows_title_options_and_help() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = ListState::default();
        state.select(Some(0));

        terminal
            .draw(|frame| {
                render_selection_dialog(
                    frame,
                    " Exec Shell ",
                    &["sh", "bash"],
                    &mut state,
                    " j/k: Navigate | Enter: Select | Esc/q: Cancel ",
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Exec Shell"));
        assert!(text.contains("sh"));
        assert!(text.contains("bash"));
        assert!(text.contains("Enter: Select"));
    }
}
