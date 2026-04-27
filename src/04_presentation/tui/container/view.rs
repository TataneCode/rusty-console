use crate::application::container::{ContainerDto, ContainerLogsDto};
use crate::domain::container::ContainerState;
use crate::presentation::tui::common::{
    filter_prompt_title, render_help, render_table, resources, split_content_area, truncate_text,
    Theme,
};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Paragraph, Row, TableState, Wrap},
    Frame,
};

pub fn render_container_list(
    frame: &mut Frame,
    area: Rect,
    containers: &[&ContainerDto],
    state: &mut TableState,
    active_filter: Option<&str>,
) {
    let [content_area, help_area] = split_content_area(area);

    let rows: Vec<Row> = containers
        .iter()
        .copied()
        .map(|c| {
            let state_style = match c.state {
                ContainerState::Running => Theme::running_style(),
                ContainerState::Paused | ContainerState::Restarting => Theme::paused_style(),
                _ => Theme::stopped_style(),
            };

            Row::new(vec![
                Cell::from(c.name.clone()),
                Cell::from(truncate_text(&c.image, 30)),
                Cell::from(c.state_display()).style(state_style),
                Cell::from(c.status.clone()),
                Cell::from(truncate_text(&c.ports, 25)),
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

    let title = filter_prompt_title(resources::CONTAINER_TITLE, active_filter);
    render_table(
        frame,
        content_area,
        &title,
        resources::CONTAINER_HEADERS.to_vec(),
        rows,
        widths,
        state,
    );

    render_help(frame, help_area, resources::CONTAINER_LIST_HELP);
}

pub fn render_container_logs(
    frame: &mut Frame,
    area: Rect,
    logs: &ContainerLogsDto,
    scroll_offset: u16,
) {
    let [content_area, help_area] = split_content_area(area);
    let title = resources::logs_title(&logs.container_name);

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

    frame.render_widget(log_content, content_area);

    render_help(frame, help_area, resources::CONTAINER_LOGS_HELP);
}

pub fn render_container_details(frame: &mut Frame, area: Rect, container: &ContainerDto) {
    let [content_area, help_area] = split_content_area(area);

    let env_section = if container.env_vars.is_empty() {
        String::new()
    } else {
        format!(
            "\n\n{}:\n{}",
            resources::CONTAINER_DETAILS_ENV_VARS_LABEL,
            container
                .env_vars
                .iter()
                .map(|v| format!("  {}", v))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    let details = format!(
        "{:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}{}",
        resources::LABEL_ID,
        container.id,
        resources::LABEL_NAME,
        container.name,
        resources::LABEL_IMAGE,
        container.image,
        resources::LABEL_STATE,
        container.state_display(),
        resources::LABEL_STATUS,
        container.status,
        resources::LABEL_CREATED,
        container.created,
        resources::LABEL_PORTS,
        container.ports,
        resources::LABEL_NETWORKS,
        container.networks,
        env_section,
    );

    let details_widget = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(resources::CONTAINER_DETAILS_TITLE)
                .title_style(Theme::title_style())
                .border_style(Theme::border_style()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(details_widget, content_area);

    render_help(frame, help_area, resources::CONTAINER_DETAILS_HELP);
}

#[cfg(test)]
mod tests {
    use super::{render_container_details, render_container_list, render_container_logs};
    use crate::application::container::{ContainerDto, ContainerLogsDto};
    use crate::domain::container::ContainerState;
    use ratatui::{backend::TestBackend, buffer::Buffer, widgets::TableState, Terminal};

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    fn make_container() -> ContainerDto {
        ContainerDto {
            id: "abc123".to_string(),
            short_id: "abc123".to_string(),
            name: "web".to_string(),
            image: "nginx:latest".to_string(),
            state: ContainerState::Running,
            status: "Up 5 minutes".to_string(),
            created: "2024-01-01".to_string(),
            ports: "80:80".to_string(),
            networks: "bridge".to_string(),
            can_start: false,
            can_stop: true,
            can_delete: true,
            can_restart: true,
            can_pause: true,
            can_unpause: false,
            env_vars: vec!["RUST_LOG=info".to_string()],
        }
    }

    #[test]
    fn test_render_container_list_shows_title_content_and_help() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let container = make_container();
        let items = vec![&container];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_container_list(frame, frame.area(), &items, &mut state, Some("ng"))
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Containers"));
        assert!(text.contains("web"));
        assert!(text.contains("Running"));
        assert!(text.contains("Logs"));
    }

    #[test]
    fn test_render_container_logs_shows_container_name_and_log_lines() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let logs = ContainerLogsDto {
            container_id: "abc123".to_string(),
            container_name: "web".to_string(),
            logs: "line1\nline2".to_string(),
        };

        terminal
            .draw(|frame| render_container_logs(frame, frame.area(), &logs, 0))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Logs: web"));
        assert!(text.contains("line1"));
        assert!(text.contains("Ctrl+u/d"));
    }

    #[test]
    fn test_render_container_details_shows_fields_and_env_vars() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let container = make_container();

        terminal
            .draw(|frame| render_container_details(frame, frame.area(), &container))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Container Details"));
        assert!(text.contains("Environment Variables"));
        assert!(text.contains("RUST_LOG=info"));
        assert!(text.contains("bridge"));
    }
}
