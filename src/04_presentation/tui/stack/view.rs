use crate::application::stack::{StackContainerDto, StackDto};
use crate::domain::stack::StackContainerState;
use crate::presentation::tui::common::{
    render_list, resources, truncate_text, ListHelpBar, ListRenderInput, ListViewSpec, Theme,
};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Cell, Row, TableState},
    Frame,
};

pub struct StackHelpBarState<'text> {
    pub is_loading: bool,
    pub spinner_char: &'text str,
}

pub fn render_stack_list(
    frame: &mut Frame,
    area: Rect,
    stacks: &[StackDto],
    state: &mut TableState,
    active_filter: Option<&str>,
    help_bar: StackHelpBarState<'_>,
) {
    let widths = vec![
        Constraint::Percentage(60),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let spec = ListViewSpec {
        title: resources::STACK_TITLE,
        headers: resources::STACK_HEADERS.to_vec(),
        widths,
        filter_enter_label: "Drill-down",
    };

    let mut input = ListRenderInput {
        items: stacks,
        state,
        active_filter,
        help_bar: ListHelpBar {
            is_loading: help_bar.is_loading,
            spinner_char: help_bar.spinner_char,
        },
    };
    render_list(frame, area, &spec, &mut input, |s| {
        let running_style = if s.running_count > 0 && s.running_count == s.container_count {
            Theme::in_use_style()
        } else {
            Theme::default_style()
        };

        Row::new(vec![
            Cell::from(s.name.clone()),
            Cell::from(s.container_count.to_string()),
            Cell::from(format!("{}/{}", s.running_count, s.container_count)).style(running_style),
        ])
    });
}

pub fn render_stack_containers(
    frame: &mut Frame,
    area: Rect,
    stack_name: &str,
    containers: &[StackContainerDto],
    state: &mut TableState,
    active_filter: Option<&str>,
    help_bar: StackHelpBarState<'_>,
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

    let title = resources::stack_containers_title(stack_name);
    let spec = ListViewSpec {
        title: &title,
        headers: resources::CONTAINER_HEADERS.to_vec(),
        widths,
        filter_enter_label: "Details",
    };

    let mut input = ListRenderInput {
        items: containers,
        state,
        active_filter,
        help_bar: ListHelpBar {
            is_loading: help_bar.is_loading,
            spinner_char: help_bar.spinner_char,
        },
    };
    render_list(frame, area, &spec, &mut input, |c| {
        let state_style = match c.state {
            StackContainerState::Running => Theme::running_style(),
            StackContainerState::Paused | StackContainerState::Restarting => Theme::paused_style(),
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

#[cfg(test)]
mod tests {
    use super::{render_stack_containers, render_stack_list, StackHelpBarState};
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
            can_restart: true,
            can_pause: true,
            can_unpause: false,
            runtime_stats: None,
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
        let items = vec![stack];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_stack_list(
                    frame,
                    frame.area(),
                    &items,
                    &mut state,
                    None,
                    StackHelpBarState {
                        is_loading: false,
                        spinner_char: "|",
                    },
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Stacks"));
        assert!(text.contains("compose-app"));
        assert!(text.contains("1/2"));
        assert!(text.contains("Enter: Drill-down"));
        assert!(text.contains("Space: All commands"));
        assert!(!text.contains("Start All"));
    }

    #[test]
    fn test_render_stack_list_shows_contextual_help_while_filter_active() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let stack = StackDto {
            name: "compose-app".to_string(),
            container_count: 2,
            running_count: 1,
            containers: vec![make_stack_container()],
        };
        let items = vec![stack];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_stack_list(
                    frame,
                    frame.area(),
                    &items,
                    &mut state,
                    Some("app"),
                    StackHelpBarState {
                        is_loading: false,
                        spinner_char: "|",
                    },
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Type: Filter"));
        assert!(text.contains("j/k: Navigate"));
        assert!(text.contains("Enter: Drill-down"));
        assert!(text.contains("Esc: Clear"));
        assert!(!text.contains("Start All"));
        assert!(!text.contains("Stop All"));
        assert!(!text.contains("Quit"));
    }

    #[test]
    fn test_render_stack_containers_shows_containers_and_help() {
        let backend = TestBackend::new(140, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let containers = vec![make_stack_container()];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_stack_containers(
                    frame,
                    frame.area(),
                    "compose-app",
                    &containers,
                    &mut state,
                    None,
                    StackHelpBarState {
                        is_loading: false,
                        spinner_char: "|",
                    },
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Stack: compose-app"));
        assert!(text.contains("web"));
        assert!(text.contains("Running"));
        assert!(text.contains("CPU"));
        assert!(text.contains("Memory"));
        assert!(text.contains("N/A"));
        assert!(text.contains("Enter: Details"));
        assert!(text.contains("Space: All commands"));
        assert!(!text.contains("Exec"));
        assert!(!text.contains("o: Sort"));
    }

    #[test]
    fn test_render_stack_containers_shows_contextual_help_while_filter_active() {
        let backend = TestBackend::new(140, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let containers = vec![make_stack_container()];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_stack_containers(
                    frame,
                    frame.area(),
                    "compose-app",
                    &containers,
                    &mut state,
                    Some("web"),
                    StackHelpBarState {
                        is_loading: false,
                        spinner_char: "|",
                    },
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("[/: web"));
        assert!(text.contains("Type: Filter"));
        assert!(text.contains("Enter: Details"));
        assert!(text.contains("Esc: Clear"));
        assert!(!text.contains("Exec"));
        assert!(!text.contains("Quit"));
    }

    #[test]
    fn test_render_stack_containers_shows_loading_bar_when_loading() {
        let backend = TestBackend::new(140, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let containers = vec![make_stack_container()];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_stack_containers(
                    frame,
                    frame.area(),
                    "compose-app",
                    &containers,
                    &mut state,
                    None,
                    StackHelpBarState {
                        is_loading: true,
                        spinner_char: "/",
                    },
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Loading..."));
        assert!(!text.contains("Exec"));
    }
}
