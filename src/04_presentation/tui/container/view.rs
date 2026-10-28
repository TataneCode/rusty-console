use crate::application::container::{ContainerDto, ContainerLogsDto, FileContentDto, FileEntryDto};
use crate::domain::container::{ContainerState, FileKind};
use crate::presentation::tui::common::{
    centered_rect, render_action_bar, render_list, resources, split_content_area, truncate_text,
    ListHelpBar, ListRenderInput, ListViewSpec, Theme,
};
use ratatui::{
    layout::{Constraint, Rect},
    text::{Line, Span},
    widgets::{Cell, Clear, List, ListItem, ListState, Paragraph, Row, TableState, Wrap},
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
    let widths = vec![
        Constraint::Percentage(16),
        Constraint::Percentage(18),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(16),
        Constraint::Percentage(18),
        Constraint::Percentage(12),
    ];

    let spec = ListViewSpec {
        title: resources::CONTAINER_TITLE,
        headers: resources::CONTAINER_HEADERS.to_vec(),
        widths,
        filter_enter_label: "Details",
    };

    let mut input = ListRenderInput {
        items: containers,
        state,
        active_filter,
        help_bar: ListHelpBar {
            is_loading,
            spinner_char,
        },
    };
    render_list(frame, area, &spec, &mut input, |c| {
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
    });
}

pub struct LogSearchView<'text> {
    pub active: bool,
    pub value: &'text str,
    pub active_term: Option<&'text str>,
    pub counter: Option<(usize, usize)>,
    pub matches: &'text [usize],
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
        .highlight_style(Theme::highlight_style())
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

/// Inputs of the container file explorer view: the entries of the current
/// directory, the container identity, the current path and the UI state.
pub struct ExplorerView<'a> {
    pub entries: &'a [FileEntryDto],
    pub state: &'a mut TableState,
    pub container_name: &'a str,
    pub path: &'a str,
    pub active_filter: Option<&'a str>,
    pub is_loading: bool,
    pub spinner_char: &'a str,
}

/// Renders the container file explorer: a table of the current directory with
/// directories highlighted in their own color, plus the contextual action bar
/// (loading / filter input / full help, same behavior as the list screens).
pub fn render_container_explorer(frame: &mut Frame, area: Rect, view: ExplorerView) {
    let ExplorerView {
        entries,
        state,
        container_name,
        path,
        active_filter,
        is_loading,
        spinner_char,
    } = view;
    let [content_area, help_area] = split_content_area(area);
    let title = resources::filter_prompt_title(
        &resources::container_explorer_title(container_name, path),
        active_filter,
    );

    let widths = vec![
        Constraint::Percentage(40),
        Constraint::Percentage(15),
        Constraint::Percentage(25),
        Constraint::Percentage(20),
    ];

    let rows: Vec<Row> = entries
        .iter()
        .map(|entry| {
            let name_style = match entry.kind {
                FileKind::Directory => Theme::directory_style(),
                FileKind::Symlink => Theme::symlink_style(),
                _ => Theme::default_style(),
            };
            Row::new(vec![
                Cell::from(Span::styled(entry.display_name(), name_style)),
                Cell::from(entry.size_display()),
                Cell::from(entry.modified_display()).style(Theme::help_style()),
                Cell::from(entry.permissions_display()).style(Theme::help_style()),
            ])
        })
        .collect();

    let headers: Vec<&str> = resources::CONTAINER_EXPLORER_HEADERS.to_vec();
    let table = ratatui::widgets::Table::new(rows, widths)
        .header(
            Row::new(
                headers
                    .into_iter()
                    .map(|h| Cell::from(h).style(Theme::header_style())),
            )
            .height(1),
        )
        .block(Theme::section_block(Some(&title)))
        .row_highlight_style(Theme::highlight_style());
    frame.render_stateful_widget(table, content_area, state);

    if is_loading {
        render_action_bar(frame, help_area, &format!(" {spinner_char} Loading... "));
    } else if active_filter.is_some() {
        render_action_bar(frame, help_area, &resources::filter_input_help("Open"));
    } else {
        render_action_bar(frame, help_area, &resources::list_footer_help("Open"));
    }
}

/// Scrollable popup showing the raw text content of a file from a container.
/// Binary content is reported as a message instead of raw bytes.
pub fn render_file_view(frame: &mut Frame, path: &str, content: &FileContentDto, scroll: usize) {
    let popup_area = centered_rect(80, 80, frame.area());
    let [content_area, help_area] = split_content_area(popup_area);
    frame.render_widget(Clear, popup_area);

    let title = format!("{}{} ", resources::FILE_VIEW_TITLE, path);
    let text = match content.text() {
        Some(text) => text,
        None => resources::file_view_binary_message(content.total_size),
    };
    let text = if content.truncated {
        format!(
            "{text}\n{}",
            resources::file_view_truncated_message(content.total_size)
        )
    } else {
        text
    };

    let content_height = content_area.height.saturating_sub(2) as usize;
    let max_scroll = text.lines().count().saturating_sub(content_height);
    let scroll = scroll.min(max_scroll);

    let popup = Paragraph::new(text)
        .block(Theme::section_block(Some(&title)))
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(popup, content_area);
    render_action_bar(frame, help_area, resources::FILE_VIEW_CLOSE_HELP);
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
        render_container_details, render_container_explorer, render_container_list,
        render_container_logs, render_file_view, ExplorerView, LogSearchView,
    };
    use crate::application::container::{
        ContainerDto, ContainerLogsDto, FileContentDto, FileEntryDto, LogLineDto,
    };
    use crate::domain::container::{ContainerState, FileKind};
    use crate::presentation::tui::common::Theme;
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        style::Modifier,
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

    fn find_text_index(buffer: &Buffer, text: &str) -> Option<usize> {
        let symbols: Vec<&str> = buffer.content.iter().map(|cell| cell.symbol()).collect();
        (0..symbols.len()).find(|&i| {
            text.chars()
                .enumerate()
                .all(|(k, ch)| symbols.get(i + k).and_then(|s| s.chars().next()) == Some(ch))
        })
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
                render_container_list(frame, frame.area(), &items, &mut state, None, false, "|")
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Containers"));
        assert!(text.contains("web"));
        assert!(text.contains("Running"));
        assert!(text.contains("12.5%"));
        assert!(text.contains("512.00 MB"));
        assert!(text.contains("j/k: Navigate"));
        assert!(text.contains("Space: All commands"));
        assert!(!text.contains("Exec"));
        assert!(!text.contains("Logs"));
    }

    #[test]
    fn test_render_container_list_shows_contextual_help_while_filter_active() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let items = vec![make_container()];
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
        assert!(text.contains("Type: Filter"));
        assert!(text.contains("j/k: Navigate"));
        assert!(text.contains("Enter: Details"));
        assert!(text.contains("Esc: Clear"));
        assert!(!text.contains("Start/Stop"));
        assert!(!text.contains("Quit"));
        assert!(!text.contains("Prune"));
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
    fn test_render_container_logs_highlights_selected_line() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let logs = ContainerLogsDto {
            container_id: "abc123".to_string(),
            container_name: "web".to_string(),
            lines: vec![
                LogLineDto {
                    timestamp: None,
                    message: "first".to_string(),
                },
                LogLineDto {
                    timestamp: None,
                    message: "second".to_string(),
                },
            ],
        };
        let mut state = ListState::default();
        state.select(Some(1));
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

        let buffer = terminal.backend().buffer();
        let highlight = Theme::highlight_style();
        for (label, selected) in [("first", false), ("second", true)] {
            let index = find_text_index(buffer, label).expect("line text present");
            let cell = &buffer.content[index];
            assert_eq!(
                Some(cell.bg) == highlight.bg,
                selected,
                "{label} line highlight background mismatch"
            );
            assert_eq!(
                cell.modifier.contains(Modifier::BOLD),
                selected,
                "{label} line bold modifier mismatch"
            );
        }
    }

    fn make_file_entry(name: &str, kind: FileKind) -> FileEntryDto {
        FileEntryDto {
            name: name.to_string(),
            path: format!("/etc/{name}"),
            kind,
            size: 1536,
            modified: Some(Utc.with_ymd_and_hms(2026, 10, 26, 10, 30, 0).unwrap()),
            permissions: if kind == FileKind::Directory {
                "drwxr-xr-x".to_string()
            } else {
                "-rw-r--r--".to_string()
            },
            link_target: None,
        }
    }

    #[test]
    fn test_render_container_explorer_shows_columns_and_help() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let entries = vec![
            make_file_entry("nginx", FileKind::Directory),
            make_file_entry("app.conf", FileKind::File),
        ];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_container_explorer(
                    frame,
                    frame.area(),
                    ExplorerView {
                        entries: &entries,
                        state: &mut state,
                        container_name: "web",
                        path: "/etc",
                        active_filter: None,
                        is_loading: false,
                        spinner_char: "|",
                    },
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Files: web /etc"));
        assert!(text.contains("Name"));
        assert!(text.contains("Size"));
        assert!(text.contains("Modified"));
        assert!(text.contains("Permissions"));
        assert!(text.contains("nginx/"));
        assert!(text.contains("app.conf"));
        assert!(text.contains("1.50 KB"));
        assert!(text.contains("2026-10-26 10:30:00"));
        assert!(text.contains("drwxr-xr-x"));
        assert!(text.contains("-rw-r--r--"));
        assert!(
            text.contains("Space: All commands"),
            "help bar missing space hint: {text}"
        );
    }

    #[test]
    fn test_render_container_explorer_directory_has_its_own_color() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let entries = vec![
            make_file_entry("nginx", FileKind::Directory),
            make_file_entry("app.conf", FileKind::File),
        ];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_container_explorer(
                    frame,
                    frame.area(),
                    ExplorerView {
                        entries: &entries,
                        state: &mut state,
                        container_name: "web",
                        path: "/",
                        active_filter: None,
                        is_loading: false,
                        spinner_char: "|",
                    },
                )
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let dir_style = Theme::directory_style();
        let file_style = Theme::default_style();
        let dir_index = find_text_index(buffer, "nginx/").expect("dir name present");
        let file_index = find_text_index(buffer, "app.conf").expect("file name present");
        assert_eq!(buffer.content[dir_index].fg, dir_style.fg.unwrap());
        assert_eq!(buffer.content[file_index].fg, file_style.fg.unwrap());
        assert_ne!(dir_style.fg, file_style.fg);
    }

    #[test]
    fn test_render_container_explorer_selected_row_is_highlighted() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let entries = vec![
            make_file_entry("nginx", FileKind::Directory),
            make_file_entry("app.conf", FileKind::File),
        ];
        let mut state = TableState::default();
        state.select(Some(1));

        terminal
            .draw(|frame| {
                render_container_explorer(
                    frame,
                    frame.area(),
                    ExplorerView {
                        entries: &entries,
                        state: &mut state,
                        container_name: "web",
                        path: "/",
                        active_filter: None,
                        is_loading: false,
                        spinner_char: "|",
                    },
                )
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let highlight = Theme::highlight_style();
        for (label, selected) in [("nginx/", false), ("app.conf", true)] {
            let index = find_text_index(buffer, label).expect("entry name present");
            let cell = &buffer.content[index];
            assert_eq!(
                Some(cell.bg) == highlight.bg,
                selected,
                "{label} row highlight background mismatch"
            );
        }
    }

    #[test]
    fn test_render_container_explorer_selected_directory_keeps_its_color() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let entries = vec![
            make_file_entry("nginx", FileKind::Directory),
            make_file_entry("app.conf", FileKind::File),
        ];
        let mut state = TableState::default();
        state.select(Some(0));

        terminal
            .draw(|frame| {
                render_container_explorer(
                    frame,
                    frame.area(),
                    ExplorerView {
                        entries: &entries,
                        state: &mut state,
                        container_name: "web",
                        path: "/",
                        active_filter: None,
                        is_loading: false,
                        spinner_char: "|",
                    },
                )
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let dir_index = find_text_index(buffer, "nginx/").expect("dir name present");
        let cell = &buffer.content[dir_index];
        assert_eq!(
            cell.bg,
            Theme::highlight_style().bg.unwrap(),
            "selected row keeps the highlight background"
        );
        assert!(
            cell.modifier.contains(Modifier::BOLD),
            "selected row is bold"
        );
        assert_eq!(
            cell.fg,
            Theme::directory_style().fg.unwrap(),
            "selected directory keeps its own color"
        );
    }

    #[test]
    fn test_render_container_explorer_filter_and_loading_bars() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let entries = vec![make_file_entry("app.conf", FileKind::File)];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_container_explorer(
                    frame,
                    frame.area(),
                    ExplorerView {
                        entries: &entries,
                        state: &mut state,
                        container_name: "web",
                        path: "/",
                        active_filter: Some("app"),
                        is_loading: false,
                        spinner_char: "|",
                    },
                )
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("[/: app"));
        assert!(text.contains("Type: Filter"));
        assert!(!text.contains("Quit"));

        terminal
            .draw(|frame| {
                render_container_explorer(
                    frame,
                    frame.area(),
                    ExplorerView {
                        entries: &entries,
                        state: &mut state,
                        container_name: "web",
                        path: "/",
                        active_filter: None,
                        is_loading: true,
                        spinner_char: "/",
                    },
                )
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Loading..."));
    }

    #[test]
    fn test_render_file_view_shows_text_content() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let content = FileContentDto {
            data: b"line1\nline2\n".to_vec(),
            total_size: 12,
            truncated: false,
        };

        terminal
            .draw(|frame| render_file_view(frame, "/etc/app.conf", &content, 0))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("File content /etc/app.conf"));
        assert!(text.contains("line1"));
        assert!(text.contains("line2"));
        assert!(!text.contains("truncated"));
    }

    #[test]
    fn test_render_file_view_reports_binary_content() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let content = FileContentDto {
            data: vec![0x00, 0x01, 0x02, 0x03],
            total_size: 4,
            truncated: false,
        };

        terminal
            .draw(|frame| render_file_view(frame, "/bin/app", &content, 0))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Binary file, 4 B"));
    }

    #[test]
    fn test_render_file_view_marks_truncated_content() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let content = FileContentDto {
            data: b"partial".to_vec(),
            total_size: 1_048_576,
            truncated: true,
        };

        terminal
            .draw(|frame| render_file_view(frame, "/var/log/app.log", &content, 0))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("partial"));
        assert!(text.contains("truncated, 1.00 MB total"));
    }

    #[test]
    fn test_render_file_view_shows_close_help_bar() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let content = FileContentDto {
            data: b"line1\n".to_vec(),
            total_size: 6,
            truncated: false,
        };

        terminal
            .draw(|frame| render_file_view(frame, "/etc/app.conf", &content, 0))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("j/k: Scroll"));
        assert!(text.contains("Esc/q: Close"));
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
