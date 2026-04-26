use crate::application::volume::VolumeDto;
use crate::presentation::tui::common::{render_help, render_table, Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Cell, Row, TableState},
    Frame,
};

pub fn render_volume_list(
    frame: &mut Frame,
    area: Rect,
    volumes: &[VolumeDto],
    state: &mut TableState,
    active_filter: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let headers = vec!["Name", "Driver", "Size", "In Use", "Created"];

    let rows: Vec<Row> = volumes
        .iter()
        .map(|v| {
            let in_use_style = if v.in_use {
                Theme::in_use_style()
            } else {
                Theme::default_style()
            };

            let in_use_text = if v.in_use { "Yes" } else { "No" };

            Row::new(vec![
                Cell::from(truncate_string(&v.name, 40)),
                Cell::from(v.driver.clone()),
                Cell::from(v.size.clone()),
                Cell::from(in_use_text).style(in_use_style),
                Cell::from(v.created.clone()),
            ])
        })
        .collect();

    let widths = vec![
        Constraint::Percentage(35),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(10),
        Constraint::Percentage(25),
    ];

    let title = match active_filter {
        Some(f) => format!(" Volumes [/: {}▏] ", f),
        None => " Volumes ".to_string(),
    };

    render_table(frame, chunks[0], &title, headers, rows, widths, state);

    render_help(
        frame,
        chunks[1],
        " q: Quit | /: Filter | j/k: Navigate | d: Delete | r: Refresh | X: Prune | Esc: Back ",
    );
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
