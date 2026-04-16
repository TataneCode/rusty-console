use crate::image::application::ImageDto;
use crate::ui::common::{render_help, render_table, Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Cell, Paragraph, Row, TableState, Wrap},
    Frame,
};

pub fn render_image_list(
    frame: &mut Frame,
    area: Rect,
    images: &[ImageDto],
    state: &mut TableState,
    active_filter: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let headers = vec!["Repository", "Tag", "ID", "Size", "In Use", "Created"];

    let rows: Vec<Row> = images
        .iter()
        .map(|img| {
            let style = if img.is_dangling {
                Theme::dangling_style()
            } else if img.in_use {
                Theme::in_use_style()
            } else {
                Theme::default_style()
            };

            let in_use_text = if img.in_use { "Yes" } else { "No" };

            Row::new(vec![
                Cell::from(truncate_string(&img.repository, 30)).style(style),
                Cell::from(img.tag.clone()).style(style),
                Cell::from(img.short_id.clone()),
                Cell::from(img.size.clone()),
                Cell::from(in_use_text).style(if img.in_use {
                    Theme::in_use_style()
                } else {
                    Theme::default_style()
                }),
                Cell::from(img.created.clone()),
            ])
        })
        .collect();

    let widths = vec![
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(12),
        Constraint::Percentage(8),
        Constraint::Percentage(25),
    ];

    let title = match active_filter {
        Some(f) => format!(" Images [/: {}▏] ", f),
        None => " Images ".to_string(),
    };

    render_table(frame, chunks[0], &title, headers, rows, widths, state);

    render_help(
        frame,
        chunks[1],
        " q: Quit | /: Filter | j/k: Navigate | d: Delete | c: Details | r: Refresh | X: Prune | Esc: Back ",
    );
}

pub fn render_image_details(frame: &mut Frame, area: Rect, image: &ImageDto) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let details = format!(
        "ID:         {}\n\
         Repository: {}\n\
         Tag:        {}\n\
         Full Name:  {}\n\
         Size:       {}\n\
         Created:    {}\n\
         In Use:     {}\n\
         Dangling:   {}",
        image.id,
        image.repository,
        image.tag,
        image.full_name,
        image.size,
        image.created,
        if image.in_use { "Yes" } else { "No" },
        if image.is_dangling { "Yes" } else { "No" },
    );

    let details_widget = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Image Details ")
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
