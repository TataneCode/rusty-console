use crate::application::{ContainerDto, ContainerLogsDto};
use crate::domain::ContainerState;
use crate::ui::common::{render_help, render_table, Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Cell, Paragraph, Row, Wrap, TableState},
    Frame,
};

pub fn render_container_list(
    frame: &mut Frame,
    area: Rect,
    containers: &[ContainerDto],
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
                ContainerState::Running => Theme::running_style(),
                ContainerState::Paused | ContainerState::Restarting => Theme::paused_style(),
                _ => Theme::stopped_style(),
            };

            Row::new(vec![
                Cell::from(c.name.clone()),
                Cell::from(truncate_string(&c.image, 30)),
                Cell::from(c.state_display()).style(state_style),
                Cell::from(c.status.clone()),
                Cell::from(truncate_string(&c.ports, 25)),
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

    render_table(
        frame,
        chunks[0],
        " Containers ",
        headers,
        rows,
        widths,
        state,
    );

    render_help(
        frame,
        chunks[1],
        " q: Quit | j/k: Navigate | l: Logs | s: Start/Stop | d: Delete | c: Details | r: Refresh ",
    );
}

pub fn render_container_logs(
    frame: &mut Frame,
    area: Rect,
    logs: &ContainerLogsDto,
    scroll_offset: u16,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let title = format!(" Logs: {} ", logs.container_name);

    let log_content = Paragraph::new(logs.logs.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(Theme::title_style())
                .border_style(Theme::border_style()),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset, 0));

    frame.render_widget(log_content, chunks[0]);

    render_help(
        frame,
        chunks[1],
        " Esc/q: Back | Ctrl+u/d: Scroll ",
    );
}

pub fn render_container_details(frame: &mut Frame, area: Rect, container: &ContainerDto) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let _state_style = match container.state {
        ContainerState::Running => Theme::running_style(),
        ContainerState::Paused | ContainerState::Restarting => Theme::paused_style(),
        _ => Theme::stopped_style(),
    };

    let details = format!(
        "ID:       {}\n\
         Name:     {}\n\
         Image:    {}\n\
         State:    {}\n\
         Status:   {}\n\
         Created:  {}\n\
         Ports:    {}\n\
         Networks: {}",
        container.id,
        container.name,
        container.image,
        container.state_display(),
        container.status,
        container.created,
        container.ports,
        container.networks,
    );

    let details_widget = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Container Details ")
                .title_style(Theme::title_style())
                .border_style(Theme::border_style()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(details_widget, chunks[0]);

    render_help(frame, chunks[1], " Esc/q: Back ");
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
