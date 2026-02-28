use crate::application::{ContainerDto, ContainerLogsDto};
use crate::ui::common::TableSelection;

pub struct ContainerPresenter {
    pub containers: Vec<ContainerDto>,
    pub selection: TableSelection,
    pub logs: Option<ContainerLogsDto>,
    pub logs_scroll: u16,
    pub selected_container: Option<ContainerDto>,
    pub loading: bool,
    pub error: Option<String>,
}

impl ContainerPresenter {
    pub fn new() -> Self {
        ContainerPresenter {
            containers: Vec::new(),
            selection: TableSelection::new(),
            logs: None,
            logs_scroll: 0,
            selected_container: None,
            loading: false,
            error: None,
        }
    }

    pub fn set_containers(&mut self, containers: Vec<ContainerDto>) {
        self.containers = containers;
        self.selection.set_items(self.containers.len());
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn selected_container(&self) -> Option<&ContainerDto> {
        self.selection
            .selected()
            .and_then(|i| self.containers.get(i))
    }

    pub fn set_logs(&mut self, logs: ContainerLogsDto) {
        self.logs = Some(logs);
        self.logs_scroll = 0;
    }

    pub fn clear_logs(&mut self) {
        self.logs = None;
        self.logs_scroll = 0;
    }

    pub fn set_details(&mut self, container: ContainerDto) {
        self.selected_container = Some(container);
    }

    pub fn clear_details(&mut self) {
        self.selected_container = None;
    }

    pub fn scroll_logs_up(&mut self, amount: u16) {
        self.logs_scroll = self.logs_scroll.saturating_sub(amount);
    }

    pub fn scroll_logs_down(&mut self, amount: u16) {
        self.logs_scroll = self.logs_scroll.saturating_add(amount);
    }

    pub fn navigate_up(&mut self) {
        self.selection.previous();
    }

    pub fn navigate_down(&mut self) {
        self.selection.next();
    }
}

impl Default for ContainerPresenter {
    fn default() -> Self {
        Self::new()
    }
}
