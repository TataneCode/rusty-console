use crate::application::container::{ContainerDto, ContainerLogsDto};
use crate::presentation::tui::common::{FilterState, TableSelection};

pub struct ContainerPresenter {
    pub containers: Vec<ContainerDto>,
    pub selection: TableSelection,
    pub logs: Option<ContainerLogsDto>,
    pub logs_scroll: u16,
    pub selected_container: Option<ContainerDto>,
    pub loading: bool,
    pub error: Option<String>,
    pub filter: FilterState,
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
            filter: FilterState::new(),
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
        if self.filter.value().is_empty() {
            self.containers.iter().collect()
        } else {
            let filter_lower = self.filter.value().to_lowercase();
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
        self.filter.activate();
    }

    pub fn deactivate_filter(&mut self) {
        self.filter.deactivate();
        self.update_filtered_selection();
    }

    pub fn push_filter_char(&mut self, c: char) {
        self.filter.push_char(c);
        self.update_filtered_selection();
    }

    pub fn pop_filter_char(&mut self) {
        self.filter.pop_char();
        self.update_filtered_selection();
    }

    pub fn is_filter_active(&self) -> bool {
        self.filter.is_active()
    }

    pub fn active_filter(&self) -> Option<&str> {
        self.filter.active_value()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::container::ContainerState;

    fn make_container(name: &str, image: &str) -> ContainerDto {
        ContainerDto {
            id: format!("id_{name}"),
            short_id: format!("id_{name}"),
            name: name.to_string(),
            image: image.to_string(),
            state: ContainerState::Running,
            status: "Up 5 minutes".to_string(),
            created: "2024-01-01".to_string(),
            ports: String::new(),
            networks: "bridge".to_string(),
            can_start: false,
            can_stop: true,
            can_delete: false,
            can_restart: true,
            can_pause: true,
            can_unpause: false,
            env_vars: vec![],
        }
    }

    fn three_containers() -> Vec<ContainerDto> {
        vec![
            make_container("alpha", "nginx:latest"),
            make_container("beta", "redis:7"),
            make_container("gamma", "nginx:alpine"),
        ]
    }

    #[test]
    fn test_new_creates_empty_state() {
        let p = ContainerPresenter::new();
        assert!(p.containers.is_empty());
        assert!(p.logs.is_none());
        assert_eq!(p.logs_scroll, 0);
        assert!(p.selected_container.is_none());
        assert!(!p.loading);
        assert!(p.error.is_none());
        assert!(p.filter.value().is_empty());
        assert!(!p.filter.is_active());
        assert!(p.selection.selected().is_none());
    }

    #[test]
    fn test_set_containers_updates_list_and_selection() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        assert_eq!(p.containers.len(), 3);
        assert_eq!(p.selection.selected(), Some(0));
    }

    #[test]
    fn test_set_containers_clears_error() {
        let mut p = ContainerPresenter::new();
        p.set_error("something failed".to_string());
        assert!(p.error.is_some());
        p.set_containers(three_containers());
        assert!(p.error.is_none());
    }

    #[test]
    fn test_filtered_containers_no_filter() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        assert_eq!(p.filtered_containers().len(), 3);
    }

    #[test]
    fn test_filtered_containers_by_name() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        p.push_filter_char('A');
        p.push_filter_char('l');
        p.push_filter_char('p');
        p.push_filter_char('h');
        let filtered = p.filtered_containers();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "alpha");
    }

    #[test]
    fn test_filtered_containers_by_image() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        p.push_filter_char('n');
        p.push_filter_char('g');
        p.push_filter_char('i');
        p.push_filter_char('n');
        p.push_filter_char('x');
        let filtered = p.filtered_containers();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filtered_containers_no_match() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        for c in "zzzzz".chars() {
            p.push_filter_char(c);
        }
        assert!(p.filtered_containers().is_empty());
    }

    #[test]
    fn test_selected_container_from_filtered_list() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        // Filter to only "redis" (beta)
        for c in "redis".chars() {
            p.push_filter_char(c);
        }
        let selected = p.selected_container();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "beta");
    }

    #[test]
    fn test_activate_deactivate_filter() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        p.activate_filter();
        assert!(p.filter.is_active());
        p.push_filter_char('a');
        assert_eq!(p.filter.value(), "a");
        p.deactivate_filter();
        assert!(!p.filter.is_active());
        assert!(p.filter.value().is_empty());
    }

    #[test]
    fn test_push_pop_filter_char() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        p.push_filter_char('a');
        p.push_filter_char('b');
        assert_eq!(p.filter.value(), "ab");
        p.pop_filter_char();
        assert_eq!(p.filter.value(), "a");
        p.pop_filter_char();
        assert!(p.filter.value().is_empty());
    }

    #[test]
    fn test_filter_updates_selection_bounds() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        // Navigate to last item (index 2)
        p.navigate_down();
        p.navigate_down();
        assert_eq!(p.selection.selected(), Some(2));
        // Filter to 1 item — selection must clamp
        for c in "beta".chars() {
            p.push_filter_char(c);
        }
        assert_eq!(p.filtered_containers().len(), 1);
        assert_eq!(p.selection.selected(), Some(0));
    }

    #[test]
    fn test_navigate_up_down() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        assert_eq!(p.selection.selected(), Some(0));
        p.navigate_down();
        assert_eq!(p.selection.selected(), Some(1));
        p.navigate_down();
        assert_eq!(p.selection.selected(), Some(2));
        // Wrap to 0
        p.navigate_down();
        assert_eq!(p.selection.selected(), Some(0));
        // Wrap backwards
        p.navigate_up();
        assert_eq!(p.selection.selected(), Some(2));
    }

    #[test]
    fn test_set_logs_and_clear() {
        let mut p = ContainerPresenter::new();
        let logs = ContainerLogsDto {
            container_id: "c1".to_string(),
            container_name: "alpha".to_string(),
            logs: "line1\nline2".to_string(),
        };
        p.set_logs(logs);
        assert!(p.logs.is_some());
        assert_eq!(p.logs_scroll, 0);
        p.clear_logs();
        assert!(p.logs.is_none());
        assert_eq!(p.logs_scroll, 0);
    }

    #[test]
    fn test_scroll_logs() {
        let mut p = ContainerPresenter::new();
        p.scroll_logs_down(5);
        assert_eq!(p.logs_scroll, 5);
        p.scroll_logs_down(3);
        assert_eq!(p.logs_scroll, 8);
        p.scroll_logs_up(2);
        assert_eq!(p.logs_scroll, 6);
        // Saturating sub — can't go below 0
        p.scroll_logs_up(100);
        assert_eq!(p.logs_scroll, 0);
    }
}
