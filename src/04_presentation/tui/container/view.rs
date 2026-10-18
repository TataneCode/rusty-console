use crate::application::container::{ContainerDto, ContainerLogsDto};
use crate::domain::container::ContainerState;
use crate::presentation::tui::common::{
    filter_prompt_title, render_action_bar, render_loading_bar, render_table, resources,
    split_content_area, truncate_text, Theme,
};
use ratatui::{
    layout::{Constraint, Rect},
    text::{Line, Span},
    widgets::{Cell, List, ListItem, ListState, Paragraph, Row, TableState, Wrap},
    Frame,
};

pub fn render_container_list(
    frame: &mut Frame,
    area: Rect,
    containers: &[ContainerDto],
    state: &mut TableState,
    active_filter: Option<&str>,
    is_loading: bool,
    spinner_char: &str,
) {
    let [content_area, help_area] = split_content_area(area);

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
                Cell::from(truncate_text(&c.image, 30)),
                Cell::from(c.state_display()).style(state_style),
                Cell::from(c.cpu_display()),
                Cell::from(truncate_text(&c.memory_display(), 18)),
                Cell::from(c.status.clone()),
                Cell::from(truncate_text(&c.ports, 25)),
            ])
        })
        .collect();

    let widths = vec![
        Constraint::Percentage(16),
        Constraint::Percentage(18),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(16),
        Constraint::Percentage(18),
        Constraint::Percentage(12),
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

    if is_loading {
        render_loading_bar(frame, help_area, spinner_char);
    } else {
        render_action_bar(frame, help_area, resources::CONTAINER_LIST_HELP);
    }
}

pub struct LogSearchView<'a> {
    pub active: bool,
    pub value: &'a str,
    pub active_term: Option<&'a str>,
    pub counter: Option<(usize, usize)>,
    pub matches: &'a [usize],
}

impl LogSearchView<'_> {
    pub fn inactive() -> Self {
        LogSearchView {
            active: false,
            value: "",
            active_term: None,
            counter: None,
            matches: &[],
        }
    }
}

pub fn render_container_logs(
    frame: &mut Frame,
    area: Rect,
    logs: &ContainerLogsDto,
    logs_state: &mut ListState,
    search: &LogSearchView,
) {
    let [content_area, help_area] = split_content_area(area);
    let title =
        resources::logs_search_title(&logs.container_name, search.active_term, search.counter);

    let items: Vec<ListItem> = logs
        .lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let timestamp = line.timestamp_display();
            let message = if search.matches.contains(&index) {
                Span::styled(line.message.clone(), Theme::highlight_style())
            } else {
                Span::from(line.message.clone())
            };
            if timestamp.is_empty() {
                ListItem::new(message)
            } else {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{timestamp} "), Theme::help_style()),
                    message,
                ]))
            }
        })
        .collect();

    let log_list = List::new(items)
        .block(Theme::section_block(Some(&title)))
        .highlight_symbol("> ");

    frame.render_stateful_widget(log_list, content_area, logs_state);

    if search.active {
        let prompt = Paragraph::new(format!("/{}{}", search.value, resources::FILTER_CURSOR))
            .style(Theme::title_style())
            .block(Theme::section_block(None));
        frame.render_widget(prompt, help_area);
    } else {
        render_action_bar(frame, help_area, resources::CONTAINER_LOGS_HELP);
    }
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
    let network_io_section = format!(
        "\n\n{}:\n  {}",
        resources::CONTAINER_DETAILS_NETWORK_IO_LABEL,
        container.network_io_display()
    );

    let details = format!(
        "{:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}\n\
         {:<10}{}\n\
         {}{}",
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
        network_io_section,
        env_section,
    );

    let details_widget = Paragraph::new(details)
        .block(Theme::section_block(Some(
            resources::CONTAINER_DETAILS_TITLE,
        )))
        .wrap(Wrap { trim: false });

    frame.render_widget(details_widget, content_area);

    render_action_bar(frame, help_area, resources::CONTAINER_DETAILS_HELP);
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use chrono::Utc;

    use super::{
        render_container_details, render_container_list, render_container_logs, LogSearchView,
    };
    use crate::application::container::{ContainerDto, ContainerLogsDto, LogLineDto};
    use crate::domain::container::ContainerState;
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        widgets::{ListState, TableState},
        Terminal,
    };

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
            runtime_stats: Some(crate::application::container::ContainerRuntimeStatsDto {
                cpu_percent: 12.5,
                memory_usage: crate::domain::shared::ByteSize::new(512 * 1024 * 1024),
                memory_limit: crate::domain::shared::ByteSize::new(1024 * 1024 * 1024),
                memory_percent: 50.0,
                network_rx: crate::domain::shared::ByteSize::new(2_048),
                network_tx: crate::domain::shared::ByteSize::new(1_024),
            }),
        }
    }

    #[test]
    fn test_render_container_list_shows_title_content_and_help() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let container = make_container();
        let items = vec![container];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_container_list(
                    frame,
                    frame.area(),
                    &items,
                    &mut state,
                    Some("ng"),
                    false,
                    "|",
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Containers"));
        assert!(text.contains("web"));
        assert!(text.contains("Running"));
        assert!(text.contains("12.5%"));
        assert!(text.contains("512.00 MB"));
        assert!(text.contains("Exec"));
        assert!(text.contains("Logs"));
    }

    #[test]
    fn test_render_container_logs_shows_container_name_and_log_lines() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let logs = ContainerLogsDto {
            container_id: "abc123".to_string(),
            container_name: "web".to_string(),
            lines: vec![
                LogLineDto {
                    timestamp: None,
                    message: "line1".to_string(),
                },
                LogLineDto {
                    timestamp: None,
                    message: "line2".to_string(),
                },
            ],
        };

        terminal
            .draw(|frame| {
                render_container_logs(
                    frame,
                    frame.area(),
                    &logs,
                    &mut ListState::default(),
                    &LogSearchView::inactive(),
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Logs: web"));
        assert!(text.contains("line1"));
        assert!(text.contains("PgUp/PgDn"));
    }

    #[test]
    fn test_render_container_logs_shows_timestamp_and_selection_symbol() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let timestamp = Utc.with_ymd_and_hms(2026, 10, 26, 10, 30, 0).unwrap();
        let logs = ContainerLogsDto {
            container_id: "abc123".to_string(),
            container_name: "web".to_string(),
            lines: vec![
                LogLineDto {
                    timestamp: Some(timestamp),
                    message: "first".to_string(),
                },
                LogLineDto {
                    timestamp: None,
                    message: "second".to_string(),
                },
            ],
        };
        let mut state = ListState::default();
        state.select(Some(0));
        terminal
            .draw(|frame| {
                render_container_logs(
                    frame,
                    frame.area(),
                    &logs,
                    &mut state,
                    &LogSearchView::inactive(),
                )
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("2026-10-26 10:30:00"));
        assert!(text.contains("first"));
        assert!(text.contains("second"));
        assert!(text.contains("> "));
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
        assert!(text.contains("Network I/O"));
        assert!(text.contains("RX 2.00 KB / TX 1.00 KB"));
        assert!(text.contains("RUST_LOG=info"));
        assert!(text.contains("bridge"));
    }

    #[test]
    fn test_render_container_logs_search_input_bar() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let logs = ContainerLogsDto {
            container_id: "abc123".to_string(),
            container_name: "web".to_string(),
            lines: vec![LogLineDto {
                timestamp: None,
                message: "line1".to_string(),
            }],
        };
        terminal
            .draw(|frame| {
                render_container_logs(
                    frame,
                    frame.area(),
                    &logs,
                    &mut ListState::default(),
                    &LogSearchView {
                        active: true,
                        value: "err",
                        active_term: None,
                        counter: None,
                        matches: &[],
                    },
                )
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("/err"));
    }

    #[test]
    fn test_render_container_logs_title_shows_search_term_and_counter() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let logs = ContainerLogsDto {
            container_id: "abc123".to_string(),
            container_name: "web".to_string(),
            lines: vec![
                LogLineDto {
                    timestamp: None,
                    message: "ok".to_string(),
                },
                LogLineDto {
                    timestamp: None,
                    message: "error here".to_string(),
                },
            ],
        };
        terminal
            .draw(|frame| {
                render_container_logs(
                    frame,
                    frame.area(),
                    &logs,
                    &mut ListState::default(),
                    &LogSearchView {
                        active: false,
                        value: "error",
                        active_term: Some("error"),
                        counter: Some((1, 1)),
                        matches: &[1],
                    },
                )
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("[/error 1/1]"));
    }
}
