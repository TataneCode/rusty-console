use crate::application::stack::{StackContainerDto, StackDto};
use crate::domain::stack::StackContainerState;
use crate::presentation::tui::common::{
    filter_prompt_title, render_help, render_table, resources, split_content_area, truncate_text,
    Theme,
};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Cell, Row, TableState},
    Frame,
};

pub fn render_stack_list(
    frame: &mut Frame,
    area: Rect,
    stacks: &[&StackDto],
    state: &mut TableState,
    active_filter: Option<&str>,
) {
    let [content_area, help_area] = split_content_area(area);

    let rows: Vec<Row> = stacks
        .iter()
        .copied()
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

    let title = filter_prompt_title(resources::STACK_TITLE, active_filter);
    render_table(
        frame,
        content_area,
        &title,
        resources::STACK_HEADERS.to_vec(),
        rows,
        widths,
        state,
    );

    render_help(frame, help_area, resources::STACK_LIST_HELP);
}

pub fn render_stack_containers(
    frame: &mut Frame,
    area: Rect,
    stack_name: &str,
    containers: &[StackContainerDto],
    state: &mut TableState,
) {
    let [content_area, help_area] = split_content_area(area);

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

    let title = resources::stack_containers_title(stack_name);
    render_table(
        frame,
        content_area,
        &title,
        resources::STACK_CONTAINER_HEADERS.to_vec(),
        rows,
        widths,
        state,
    );

    render_help(frame, help_area, resources::STACK_CONTAINERS_HELP);
}

#[cfg(test)]
mod tests {
    use super::{render_stack_containers, render_stack_list};
    use crate::application::stack::{StackContainerDto, StackDto};
    use crate::domain::stack::StackContainerState;
    use ratatui::{backend::TestBackend, buffer::Buffer, widgets::TableState, Terminal};

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    fn make_stack_container() -> StackContainerDto {
        StackContainerDto {
            id: "1".to_string(),
            name: "web".to_string(),
            image: "nginx:latest".to_string(),
            state: StackContainerState::Running,
            status: "Up".to_string(),
            ports: "80/tcp".to_string(),
            can_start: false,
            can_stop: true,
        }
    }

    #[test]
    fn test_render_stack_list_shows_stack_summary() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let stack = StackDto {
            name: "compose-app".to_string(),
            container_count: 2,
            running_count: 1,
            containers: vec![make_stack_container()],
        };
        let items = vec![&stack];
        let mut state = TableState::default();

        terminal
            .draw(|frame| render_stack_list(frame, frame.area(), &items, &mut state, Some("app")))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Stacks"));
        assert!(text.contains("compose-app"));
        assert!(text.contains("1/2"));
        assert!(text.contains("Start All"));
    }

    #[test]
    fn test_render_stack_containers_shows_containers_and_help() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let containers = vec![make_stack_container()];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_stack_containers(frame, frame.area(), "compose-app", &containers, &mut state)
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Stack: compose-app"));
        assert!(text.contains("web"));
        assert!(text.contains("Running"));
        assert!(text.contains("Remove All"));
    }
}
