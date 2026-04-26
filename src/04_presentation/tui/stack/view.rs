use crate::application::stack::{StackContainerDto, StackDto};
use crate::domain::stack::StackContainerState;
use crate::presentation::tui::common::{render_help, render_table, Theme};
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

pub fn render_stack_containers(
    frame: &mut Frame,
    area: Rect,
    stack_name: &str,
    containers: &[StackContainerDto],
    state: &mut TableState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let headers = vec!["Name", "Image", "State", "Status", "Ports"];

    let rows: Vec<Row> = containers
        .iter()
        .map(|c| {
            let state_style = match c.state {
                StackContainerState::Running => Theme::running_style(),
                StackContainerState::Paused | StackContainerState::Restarting => {
                    Theme::paused_style()
                }
                _ => Theme::stopped_style(),
            };

            let image = if c.image.chars().count() > 30 {
                format!("{}…", c.image.chars().take(29).collect::<String>())
            } else {
                c.image.clone()
            };
            let ports = if c.ports.chars().count() > 25 {
                format!("{}…", c.ports.chars().take(24).collect::<String>())
            } else {
                c.ports.clone()
            };

            Row::new(vec![
                Cell::from(c.name.clone()),
                Cell::from(image),
                Cell::from(c.state_display()).style(state_style),
                Cell::from(c.status.clone()),
                Cell::from(ports),
            ])
        })
        .collect();

    let widths = vec![
        Constraint::Percentage(20),
        Constraint::Percentage(25),
        Constraint::Percentage(10),
        Constraint::Percentage(25),
        Constraint::Percentage(20),
    ];

    let title = format!(" Stack: {} ", stack_name);
    render_table(frame, chunks[0], &title, headers, rows, widths, state);

    render_help(
        frame,
        chunks[1],
        " Esc/q: Back | j/k: Navigate | s: Start/Stop | S: Stop All | Ctrl+S: Start All | D: Remove All | d: Delete | r: Refresh ",
    );
}
