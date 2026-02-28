use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use super::theme::Theme;

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
    let area = centered_rect(50, 20, frame.area());

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
        "{}\n\n  [Yes]  [No]",
        message
    );

    let dialog = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm ")
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

    let yes_text = Paragraph::new("  [Yes]").style(yes_style);
    let no_text = Paragraph::new("  [No]").style(no_style);

    let yes_area = Rect {
        x: button_area.x,
        y: button_area.y,
        width: 7,
        height: 1,
    };

    let no_area = Rect {
        x: button_area.x + 9,
        y: button_area.y,
        width: 6,
        height: 1,
    };

    frame.render_widget(yes_text, yes_area);
    frame.render_widget(no_text, no_area);
}

pub fn render_error_popup(frame: &mut Frame, error: &str) {
    let area = centered_rect(60, 20, frame.area());

    frame.render_widget(Clear, area);

    let error_widget = Paragraph::new(error)
        .style(Theme::error_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Error ")
                .title_style(Theme::error_style())
                .border_style(Theme::error_style()),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(error_widget, area);
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
