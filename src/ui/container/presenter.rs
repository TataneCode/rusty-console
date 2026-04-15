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
    pub filter: String,
    pub filter_active: bool,
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
            filter: String::new(),
            filter_active: false,
        }
    }

    pub fn set_containers(&mut self, containers: Vec<ContainerDto>) {
        self.containers = containers;
        self.update_filtered_selection();
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn filtered_containers(&self) -> Vec<&ContainerDto> {
        if self.filter.is_empty() {
            self.containers.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.containers
                .iter()
                .filter(|c| {
                    c.name.to_lowercase().contains(&filter_lower)
                        || c.image.to_lowercase().contains(&filter_lower)
                })
                .collect()
        }
    }

    pub fn selected_container(&self) -> Option<&ContainerDto> {
        let filtered = self.filtered_containers();
        self.selection
            .selected()
            .and_then(|i| filtered.get(i).copied())
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

    pub fn activate_filter(&mut self) {
        self.filter_active = true;
    }

    pub fn deactivate_filter(&mut self) {
        self.filter_active = false;
        self.filter.clear();
        self.update_filtered_selection();
    }

    pub fn push_filter_char(&mut self, c: char) {
        self.filter.push(c);
        self.update_filtered_selection();
    }

    pub fn pop_filter_char(&mut self) {
        self.filter.pop();
        self.update_filtered_selection();
    }

    fn update_filtered_selection(&mut self) {
        let count = self.filtered_containers().len();
        self.selection.set_items(count);
    }
}

impl Default for ContainerPresenter {
    fn default() -> Self {
        Self::new()
    }
}
