use crate::stack::application::StackDto;
use crate::ui::common::{render_help, render_table, Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Cell, Row, TableState},
    Frame,
};

pub fn render_stack_list(
    frame: &mut Frame,
    area: Rect,
    stacks: &[StackDto],
    state: &mut TableState,
    active_filter: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let headers = vec!["Stack", "Containers", "Running"];

    let rows: Vec<Row> = stacks
        .iter()
        .map(|s| {
            let running_style = if s.running_count > 0 && s.running_count == s.container_count {
                Theme::in_use_style()
            } else {
                Theme::default_style()
            };

            Row::new(vec![
                Cell::from(s.name.clone()),
                Cell::from(s.container_count.to_string()),
                Cell::from(format!("{}/{}", s.running_count, s.container_count))
                    .style(running_style),
            ])
        })
        .collect();

    let widths = vec![
        Constraint::Percentage(60),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let title = match active_filter {
        Some(f) => format!(" Stacks [/: {}▏] ", f),
        None => " Stacks ".to_string(),
    };

    render_table(frame, chunks[0], &title, headers, rows, widths, state);

    render_help(
        frame,
        chunks[1],
        " q: Quit | /: Filter | j/k: Navigate | Enter: Drill-down | s: Start All | S: Stop All | r: Refresh | Esc: Back ",
    );
}
