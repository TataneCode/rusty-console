use crate::application::volume::VolumeDto;
use crate::presentation::tui::common::{
    filter_prompt_title, render_help, render_table, resources, split_content_area, truncate_text,
    Theme,
};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Paragraph, Row, TableState, Wrap},
    Frame,
};

pub fn render_volume_list(
    frame: &mut Frame,
    area: Rect,
    volumes: &[&VolumeDto],
    state: &mut TableState,
    active_filter: Option<&str>,
) {
    let [content_area, help_area] = split_content_area(area);

    let rows: Vec<Row> = volumes
        .iter()
        .copied()
        .map(|v| {
            let in_use_style = if v.in_use {
                Theme::in_use_style()
            } else {
                Theme::default_style()
            };

            let in_use_text = if v.in_use {
                resources::VALUE_YES
            } else {
                resources::VALUE_NO
            };

            Row::new(vec![
                Cell::from(truncate_text(&v.name, 40)),
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

    let title = filter_prompt_title(resources::VOLUME_TITLE, active_filter);
    render_table(
        frame,
        content_area,
        &title,
        resources::VOLUME_HEADERS.to_vec(),
        rows,
        widths,
        state,
    );

    render_help(frame, help_area, resources::VOLUME_LIST_HELP);
}

pub fn render_volume_details(frame: &mut Frame, area: Rect, volume: &VolumeDto) {
    let [content_area, help_area] = split_content_area(area);

    let linked_containers = if volume.linked_containers.is_empty() {
        resources::VALUE_NO.to_string()
    } else {
        volume
            .linked_containers
            .iter()
            .map(|name| format!("  {}", name))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let details = format!(
        "{:<18}{}\n\
         {:<18}{}\n\
         {:<18}{}\n\
         {:<18}{}\n\
         {:<18}{}\n\
         {:<18}{}\n\
         {:<18}{}\n\
         {}:\n{}",
        resources::LABEL_ID,
        volume.id.as_str(),
        resources::LABEL_NAME,
        volume.name.as_str(),
        resources::LABEL_DRIVER,
        volume.driver.as_str(),
        resources::LABEL_MOUNTPOINT,
        volume.mountpoint.as_str(),
        resources::LABEL_SIZE,
        volume.size.as_str(),
        resources::LABEL_CREATED,
        volume.created.as_str(),
        resources::LABEL_IN_USE,
        if volume.in_use {
            resources::VALUE_YES
        } else {
            resources::VALUE_NO
        },
        resources::LABEL_LINKED_CONTAINERS.trim_end_matches(':'),
        linked_containers,
    );

    let details_widget = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(resources::VOLUME_DETAILS_TITLE)
                .title_style(Theme::title_style())
                .border_style(Theme::border_style()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(details_widget, content_area);

    render_help(frame, help_area, resources::VOLUME_DETAILS_HELP);
}

#[cfg(test)]
mod tests {
    use super::{render_volume_details, render_volume_list};
    use crate::application::volume::VolumeDto;
    use ratatui::{backend::TestBackend, buffer::Buffer, widgets::TableState, Terminal};

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    #[test]
    fn test_render_volume_list_shows_volume_data_and_help() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let volume = VolumeDto {
            id: "vol-1".to_string(),
            name: "db-data".to_string(),
            driver: "local".to_string(),
            mountpoint: "/var/lib/docker/volumes/db-data/_data".to_string(),
            size: "10 MB".to_string(),
            created: "2024-01-01".to_string(),
            in_use: true,
            linked_containers: vec!["/web".to_string()],
            can_delete: false,
        };
        let items = vec![&volume];
        let mut state = TableState::default();

        terminal
            .draw(|frame| render_volume_list(frame, frame.area(), &items, &mut state, Some("db")))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Volumes"));
        assert!(text.contains("db-data"));
        assert!(text.contains("local"));
        assert!(text.contains("Prune"));
    }

    #[test]
    fn test_render_volume_details_shows_full_name_and_linked_containers() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let volume = VolumeDto {
            id: "vol-1".to_string(),
            name: "super-long-volume-name-for-details".to_string(),
            driver: "local".to_string(),
            mountpoint: "/var/lib/docker/volumes/db-data/_data".to_string(),
            size: "10 MB".to_string(),
            created: "2024-01-01".to_string(),
            in_use: true,
            linked_containers: vec!["/web".to_string(), "/worker".to_string()],
            can_delete: false,
        };

        terminal
            .draw(|frame| render_volume_details(frame, frame.area(), &volume))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Volume Details"));
        assert!(text.contains("super-long-volume-name-for-details"));
        assert!(text.contains("Linked Containers"));
        assert!(text.contains("/web"));
        assert!(text.contains("/worker"));
    }
}
