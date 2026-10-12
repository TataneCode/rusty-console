use crate::application::network::NetworkDto;
use crate::presentation::tui::common::{
    filter_prompt_title, render_help, render_table, resources, split_content_area, truncate_text,
    Theme,
};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Paragraph, Row, TableState, Wrap},
    Frame,
};

pub fn render_network_list(
    frame: &mut Frame,
    area: Rect,
    networks: &[&NetworkDto],
    state: &mut TableState,
    active_filter: Option<&str>,
    is_loading: bool,
    spinner_char: &str,
) {
    let [content_area, help_area] = split_content_area(area);

    let rows: Vec<Row> = networks
        .iter()
        .copied()
        .map(|n| {
            let builtin_style = if n.builtin {
                Theme::in_use_style()
            } else {
                Theme::default_style()
            };

            let subnet = n.subnet.clone().unwrap_or_else(|| "N/A".to_string());

            Row::new(vec![
                Cell::from(truncate_text(&n.name, 40)),
                Cell::from(n.driver.clone()),
                Cell::from(n.scope.clone()),
                Cell::from(subnet),
                Cell::from(n.container_count.to_string()).style(builtin_style),
            ])
        })
        .collect();

    let widths = vec![
        Constraint::Percentage(35),
        Constraint::Percentage(15),
        Constraint::Percentage(10),
        Constraint::Percentage(25),
        Constraint::Percentage(15),
    ];

    let title = filter_prompt_title(resources::NETWORK_TITLE, active_filter);
    render_table(
        frame,
        content_area,
        &title,
        resources::NETWORK_HEADERS.to_vec(),
        rows,
        widths,
        state,
    );

    let help_text = if is_loading {
        format!(" {} Loading...", spinner_char)
    } else {
        resources::NETWORK_LIST_HELP.to_string()
    };
    render_help(frame, help_area, &help_text);
}

pub fn render_network_details(
    frame: &mut Frame,
    area: Rect,
    network: &NetworkDto,
    selected_container: Option<usize>,
) {
    let [content_area, help_area] = split_content_area(area);

    let subnet = network.subnet.clone().unwrap_or_else(|| "N/A".to_string());
    let gateway = network.gateway.clone().unwrap_or_else(|| "N/A".to_string());

    let containers = if network.containers.is_empty() {
        resources::VALUE_NO.to_string()
    } else {
        network
            .containers
            .iter()
            .enumerate()
            .map(|(index, container)| {
                let marker = if selected_container == Some(index) {
                    "  >"
                } else {
                    ""
                };
                format!(
                    "{marker} {} ({})",
                    container.name, container.ip
                )
            })
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
         {:<18}{}\n\
         {:<18}{}\n\
         {}:\n{}",
        resources::LABEL_ID,
        network.id.as_str(),
        resources::LABEL_NAME,
        network.name.as_str(),
        resources::LABEL_DRIVER,
        network.driver.as_str(),
        resources::LABEL_SCOPE,
        network.scope.as_str(),
        resources::LABEL_INTERNAL,
        if network.internal {
            resources::VALUE_YES
        } else {
            resources::VALUE_NO
        },
        resources::LABEL_ATTACHABLE,
        if network.attachable {
            resources::VALUE_YES
        } else {
            resources::VALUE_NO
        },
        resources::LABEL_SUBNET,
        subnet.as_str(),
        resources::LABEL_GATEWAY,
        gateway.as_str(),
        resources::LABEL_CREATED,
        network.created.as_str(),
        resources::LABEL_CONTAINERS.trim_end_matches(':'),
        containers,
    );

    let details_widget = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(resources::NETWORK_DETAILS_TITLE)
                .title_style(Theme::title_style())
                .border_style(Theme::border_style()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(details_widget, content_area);

    render_help(frame, help_area, resources::NETWORK_DETAILS_HELP);
}

#[cfg(test)]
mod tests {
    use super::{render_network_details, render_network_list};
    use crate::application::network::{NetworkContainerDto, NetworkDto};
    use ratatui::{backend::TestBackend, buffer::Buffer, widgets::TableState, Terminal};

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    fn make_network_dto() -> NetworkDto {
        NetworkDto {
            id: "net-1".to_string(),
            name: "app-net".to_string(),
            driver: "bridge".to_string(),
            scope: "local".to_string(),
            internal: true,
            attachable: false,
            subnet: Some("172.20.0.0/16".to_string()),
            gateway: Some("172.20.0.1".to_string()),
            created: "2024-01-01".to_string(),
            builtin: false,
            containers: vec![NetworkContainerDto {
                name: "web".to_string(),
                ip: "172.20.0.2/16".to_string(),
            }],
            can_delete: false,
            container_count: 1,
        }
    }

    #[test]
    fn test_render_network_list_shows_network_data_and_help() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let network = make_network_dto();
        let items = vec![&network];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_network_list(
                    frame,
                    frame.area(),
                    &items,
                    &mut state,
                    Some("app"),
                    false,
                    "|",
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Networks"));
        assert!(text.contains("app-net"));
        assert!(text.contains("bridge"));
        assert!(text.contains("172.20.0.0/16"));
        assert!(text.contains("Prune"));
    }

    #[test]
    fn test_render_network_list_shows_loading_spinner() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let network = make_network_dto();
        let items = vec![&network];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_network_list(frame, frame.area(), &items, &mut state, None, true, "/")
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Loading..."));
        assert!(!text.contains("Prune"));
    }

    #[test]
    fn test_render_network_details_shows_sections_and_containers() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let network = make_network_dto();

        terminal
            .draw(|frame| {
                render_network_details(frame, frame.area(), &network, Some(0))
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Network Details"));
        assert!(text.contains("app-net"));
        assert!(text.contains("172.20.0.0/16"));
        assert!(text.contains("172.20.0.1"));
        assert!(text.contains("web (172.20.0.2/16)"));
        assert!(text.contains("Disconnect"));
    }

    #[test]
    fn test_render_network_details_marks_selected_container() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let network = make_network_dto();

        terminal
            .draw(|frame| {
                render_network_details(frame, frame.area(), &network, Some(0))
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("> web ("));
    }

    #[test]
    fn test_render_network_details_without_containers_shows_no() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut network = make_network_dto();
        network.containers = Vec::new();
        network.container_count = 0;

        terminal
            .draw(|frame| render_network_details(frame, frame.area(), &network, None))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(!text.contains("web ("));
    }
}
