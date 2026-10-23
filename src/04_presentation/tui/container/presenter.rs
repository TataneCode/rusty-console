use crate::application::container::{
    ContainerDto, ContainerLogsDto, ContainerRuntimeStatsDto, ContainerStatsUpdate,
};
use crate::presentation::tui::common::{FilterState, ListCore};
use ratatui::widgets::ListState;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

pub struct ContainerPresenter {
    pub list: ListCore<ContainerDto>,
    pub logs: Option<ContainerLogsDto>,
    pub logs_state: ListState,
    pub selected_container: Option<ContainerDto>,
    pub log_search: FilterState,
    pub log_search_results: Vec<usize>,
    pub log_search_position: usize,
    stats_by_container_id: HashMap<String, ContainerRuntimeStatsDto>,
}

fn matches_container(container: &ContainerDto, filter_lower: &str) -> bool {
    container.name.to_lowercase().contains(filter_lower)
        || container.image.to_lowercase().contains(filter_lower)
}

fn container_sort_key(container: &ContainerDto) -> &str {
    container.name.as_str()
}

impl ContainerPresenter {
    pub fn new() -> Self {
        ContainerPresenter {
            list: ListCore::new(matches_container, container_sort_key),
            logs: None,
            logs_state: ListState::default(),
            selected_container: None,
            log_search: FilterState::new(),
            log_search_results: Vec::new(),
            log_search_position: 0,
            stats_by_container_id: HashMap::new(),
        }
    }

    pub fn set_containers(&mut self, containers: Vec<ContainerDto>) {
        self.list.set_items(containers);
        self.reapply_runtime_stats();
    }

    pub fn filtered_containers(&self) -> Vec<&ContainerDto> {
        self.list.filtered_items()
    }

    pub fn selected_container(&self) -> Option<&ContainerDto> {
        self.list.selected_item()
    }

    pub fn set_logs(&mut self, logs: ContainerLogsDto) {
        self.logs = Some(logs);
        self.logs_state = ListState::default();
        if !self.logs.as_ref().unwrap().lines.is_empty() {
            self.logs_state.select(Some(0));
        }
        self.log_search_results.clear();
        self.log_search_position = 0;
    }

    pub fn clear_logs(&mut self) {
        self.logs = None;
        self.logs_state = ListState::default();
    }

    pub fn set_details(&mut self, container: ContainerDto) {
        let mut container = container;
        container.runtime_stats = self.stats_by_container_id.get(&container.id).cloned();
        self.selected_container = Some(container);
    }

    pub fn clear_details(&mut self) {
        self.selected_container = None;
    }

    pub fn next_logs_line(&mut self) {
        let len = self.logs_lines_len();
        if len == 0 {
            return;
        }
        let current = self.logs_state.selected().unwrap_or(0);
        let next = if current + 1 >= len { 0 } else { current + 1 };
        self.logs_state.select(Some(next));
    }

    pub fn previous_logs_line(&mut self) {
        let len = self.logs_lines_len();
        if len == 0 {
            return;
        }
        let current = self.logs_state.selected().unwrap_or(0);
        let previous = if current == 0 { len - 1 } else { current - 1 };
        self.logs_state.select(Some(previous));
    }

    pub fn next_logs_page(&mut self, amount: usize) {
        let len = self.logs_lines_len();
        if len == 0 {
            return;
        }
        let current = self.logs_state.selected().unwrap_or(0);
        let next = (current + amount).min(len - 1);
        self.logs_state.select(Some(next));
    }

    pub fn previous_logs_page(&mut self, amount: usize) {
        let len = self.logs_lines_len();
        if len == 0 {
            return;
        }
        let current = self.logs_state.selected().unwrap_or(0);
        let previous = current.saturating_sub(amount);
        self.logs_state.select(Some(previous));
    }

    pub fn select_logs_line(&mut self, index: usize) {
        let len = self.logs_lines_len();
        if len == 0 {
            return;
        }
        self.logs_state.select(Some(index.min(len - 1)));
    }

    pub fn selected_logs_line(&self) -> Option<usize> {
        self.logs_state.selected()
    }

    pub fn logs_lines_len(&self) -> usize {
        self.logs.as_ref().map(|logs| logs.lines.len()).unwrap_or(0)
    }

    pub fn is_log_search_active(&self) -> bool {
        self.log_search.is_active()
    }

    pub fn activate_log_search(&mut self) {
        self.log_search.deactivate();
        self.log_search.activate();
        self.log_search_results.clear();
        self.log_search_position = 0;
    }

    pub fn deactivate_log_search(&mut self) {
        self.log_search.deactivate();
        self.log_search_results.clear();
        self.log_search_position = 0;
    }

    pub fn log_search_value(&self) -> &str {
        self.log_search.value()
    }

    pub fn push_log_search_char(&mut self, c: char) {
        self.log_search.push_char(c);
    }

    pub fn pop_log_search_char(&mut self) {
        self.log_search.pop_char();
    }

    pub fn confirm_log_search(&mut self) {
        let term = self.log_search.value().to_lowercase();
        self.log_search_results = self
            .logs
            .iter()
            .flat_map(|logs| logs.lines.iter())
            .enumerate()
            .filter(|(_, line)| !term.is_empty() && line.message.to_lowercase().contains(&term))
            .map(|(index, _)| index)
            .collect();
        self.log_search_position = 0;
        self.log_search.stop_input();
        self.jump_to_current_match();
    }

    pub fn next_log_match(&mut self) {
        if self.log_search_results.is_empty() {
            return;
        }
        self.log_search_position = (self.log_search_position + 1) % self.log_search_results.len();
        self.jump_to_current_match();
    }

    pub fn previous_log_match(&mut self) {
        if self.log_search_results.is_empty() {
            return;
        }
        self.log_search_position = if self.log_search_position == 0 {
            self.log_search_results.len() - 1
        } else {
            self.log_search_position - 1
        };
        self.jump_to_current_match();
    }

    pub fn active_search_value(&self) -> Option<&str> {
        (!self.log_search_results.is_empty()).then_some(self.log_search.value())
    }

    pub fn search_counter(&self) -> Option<(usize, usize)> {
        (!self.log_search_results.is_empty())
            .then(|| (self.log_search_position + 1, self.log_search_results.len()))
    }

    pub fn is_log_match(&self, index: usize) -> bool {
        self.log_search_results.contains(&index)
    }

    fn jump_to_current_match(&mut self) {
        if let Some(&index) = self.log_search_results.get(self.log_search_position) {
            self.select_logs_line(index);
        }
    }

    pub fn apply_stats_update(&mut self, update: ContainerStatsUpdate) {
        self.stats_by_container_id
            .insert(update.container_id.clone(), update.stats.clone());

        if let Some(container) = self
            .list
            .items
            .iter_mut()
            .find(|container| container.id == update.container_id)
        {
            container.runtime_stats = Some(update.stats.clone());
        }

        if let Some(container) = self.selected_container.as_mut() {
            if container.id == update.container_id {
                container.runtime_stats = Some(update.stats);
            }
        }
    }

    pub fn retain_runtime_stats(&mut self, monitored_container_ids: &HashSet<String>) {
        self.stats_by_container_id
            .retain(|id, _| monitored_container_ids.contains(id));
        self.reapply_runtime_stats();
    }

    pub fn clear_runtime_stats(&mut self) {
        self.stats_by_container_id.clear();
        self.reapply_runtime_stats();
    }

    fn reapply_runtime_stats(&mut self) {
        for container in &mut self.list.items {
            container.runtime_stats = self.stats_by_container_id.get(&container.id).cloned();
        }

        if let Some(container) = self.selected_container.as_mut() {
            container.runtime_stats = self.stats_by_container_id.get(&container.id).cloned();
        }
    }
}

impl Deref for ContainerPresenter {
    type Target = ListCore<ContainerDto>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for ContainerPresenter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
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
    use crate::application::container::LogLineDto;
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
            runtime_stats: None,
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
        assert!(p.items.is_empty());
        assert!(p.logs.is_none());
        assert!(p.logs_state.selected().is_none());
        assert!(p.selected_container.is_none());
        assert!(!p.loading);
        assert!(p.error.is_none());
        assert!(p.filter.value().is_empty());
        assert!(!p.filter.is_active());
        assert!(p.selection.selected().is_none());
        assert!(p.stats_by_container_id.is_empty());
    }

    #[test]
    fn test_set_containers_updates_list_and_selection() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        assert_eq!(p.items.len(), 3);
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
            lines: vec![LogLineDto {
                timestamp: None,
                message: "line1".to_string(),
            }],
        };
        p.set_logs(logs);
        assert!(p.logs.is_some());
        assert_eq!(p.logs_state.selected(), Some(0));
        p.clear_logs();
        assert!(p.logs.is_none());
        assert!(p.logs_state.selected().is_none());
    }

    fn make_logs(lines: usize) -> ContainerLogsDto {
        ContainerLogsDto {
            container_id: "c1".to_string(),
            container_name: "alpha".to_string(),
            lines: (0..lines)
                .map(|i| LogLineDto {
                    timestamp: None,
                    message: format!("line{i}"),
                })
                .collect(),
        }
    }

    #[test]
    fn test_next_previous_logs_line_wrap_around() {
        let mut p = ContainerPresenter::new();
        p.set_logs(make_logs(3));
        assert_eq!(p.logs_state.selected(), Some(0));
        p.next_logs_line();
        assert_eq!(p.logs_state.selected(), Some(1));
        p.next_logs_line();
        assert_eq!(p.logs_state.selected(), Some(2));
        p.next_logs_line();
        assert_eq!(p.logs_state.selected(), Some(0));
        p.previous_logs_line();
        assert_eq!(p.logs_state.selected(), Some(2));
        p.previous_logs_line();
        assert_eq!(p.logs_state.selected(), Some(1));
    }

    #[test]
    fn test_next_previous_logs_page_clamps_at_bounds() {
        let mut p = ContainerPresenter::new();
        p.set_logs(make_logs(30));
        p.next_logs_page(10);
        assert_eq!(p.logs_state.selected(), Some(10));
        p.next_logs_page(10);
        assert_eq!(p.logs_state.selected(), Some(20));
        p.next_logs_page(10);
        assert_eq!(p.logs_state.selected(), Some(29));
        p.previous_logs_page(25);
        assert_eq!(p.logs_state.selected(), Some(4));
        p.previous_logs_page(100);
        assert_eq!(p.logs_state.selected(), Some(0));
    }

    #[test]
    fn test_logs_navigation_no_logs_is_noop() {
        let mut p = ContainerPresenter::new();
        p.next_logs_line();
        p.previous_logs_line();
        p.next_logs_page(10);
        p.previous_logs_page(10);
        assert!(p.logs_state.selected().is_none());
    }

    #[test]
    fn test_select_logs_line_clamps_index() {
        let mut p = ContainerPresenter::new();
        p.set_logs(make_logs(3));
        p.select_logs_line(100);
        assert_eq!(p.logs_state.selected(), Some(2));
        p.select_logs_line(0);
        assert_eq!(p.selected_logs_line(), Some(0));
    }

    #[test]
    fn test_set_logs_with_empty_lines_has_no_selection() {
        let mut p = ContainerPresenter::new();
        p.set_logs(make_logs(0));
        assert!(p.logs_state.selected().is_none());
        assert_eq!(p.logs_lines_len(), 0);
    }

    #[test]
    fn test_apply_stats_update_updates_list_and_selected_details() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        let details = p.items[0].clone();
        p.set_details(details);

        p.apply_stats_update(ContainerStatsUpdate {
            container_id: "id_alpha".to_string(),
            stats: ContainerRuntimeStatsDto {
                cpu_percent: 12.5,
                memory_usage: crate::domain::shared::ByteSize::new(512),
                memory_limit: crate::domain::shared::ByteSize::new(1024),
                memory_percent: 50.0,
                network_rx: crate::domain::shared::ByteSize::new(256),
                network_tx: crate::domain::shared::ByteSize::new(128),
            },
        });

        assert_eq!(
            p.items[0].runtime_stats.as_ref().unwrap().cpu_display(),
            "12.5%"
        );
        assert_eq!(
            p.selected_container
                .as_ref()
                .unwrap()
                .runtime_stats
                .as_ref()
                .unwrap()
                .memory_list_display(),
            "512 B (50%)"
        );
    }

    #[test]
    fn test_retain_and_clear_runtime_stats_remove_cached_values() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        p.apply_stats_update(ContainerStatsUpdate {
            container_id: "id_alpha".to_string(),
            stats: ContainerRuntimeStatsDto {
                cpu_percent: 1.0,
                memory_usage: crate::domain::shared::ByteSize::new(10),
                memory_limit: crate::domain::shared::ByteSize::new(20),
                memory_percent: 50.0,
                network_rx: crate::domain::shared::ByteSize::new(1),
                network_tx: crate::domain::shared::ByteSize::new(2),
            },
        });

        p.retain_runtime_stats(&HashSet::from(["id_beta".to_string()]));
        assert!(p.items[0].runtime_stats.is_none());

        p.apply_stats_update(ContainerStatsUpdate {
            container_id: "id_beta".to_string(),
            stats: ContainerRuntimeStatsDto {
                cpu_percent: 2.0,
                memory_usage: crate::domain::shared::ByteSize::new(20),
                memory_limit: crate::domain::shared::ByteSize::new(40),
                memory_percent: 50.0,
                network_rx: crate::domain::shared::ByteSize::new(3),
                network_tx: crate::domain::shared::ByteSize::new(4),
            },
        });
        p.clear_runtime_stats();
        assert!(p
            .items
            .iter()
            .all(|container| container.runtime_stats.is_none()));
    }

    #[test]
    fn test_set_loading_changes_state() {
        let mut p = ContainerPresenter::new();
        assert!(!p.is_loading());

        p.set_loading(true);
        assert!(p.is_loading());

        p.set_loading(false);
        assert!(!p.is_loading());
    }

    #[test]
    fn test_toggle_sort_cycles_orders() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_containers()
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_containers()
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(names, vec!["gamma", "beta", "alpha"]);
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_containers()
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_sort_survives_set_containers_refresh() {
        let mut p = ContainerPresenter::new();
        p.set_containers(three_containers());
        p.toggle_sort();
        p.set_containers(three_containers());
        let names: Vec<&str> = p
            .filtered_containers()
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_loading_state_defaults_to_false() {
        let p = ContainerPresenter::new();
        assert!(!p.is_loading());
        assert!(!p.loading);
    }

    fn make_searchable_logs() -> ContainerLogsDto {
        ContainerLogsDto {
            container_id: "c1".to_string(),
            container_name: "alpha".to_string(),
            lines: vec![
                LogLineDto {
                    timestamp: None,
                    message: "starting app".to_string(),
                },
                LogLineDto {
                    timestamp: None,
                    message: "ERROR disk full".to_string(),
                },
                LogLineDto {
                    timestamp: None,
                    message: "still running".to_string(),
                },
                LogLineDto {
                    timestamp: None,
                    message: "error retrying".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_log_search_case_insensitive_confirm_jumps_to_first_match() {
        let mut p = ContainerPresenter::new();
        p.set_logs(make_searchable_logs());
        p.activate_log_search();
        for c in "error".chars() {
            p.push_log_search_char(c);
        }
        p.confirm_log_search();
        assert_eq!(p.log_search_results, vec![1, 3]);
        assert_eq!(p.selected_logs_line(), Some(1));
        assert_eq!(p.search_counter(), Some((1, 2)));
        assert_eq!(p.active_search_value(), Some("error"));
    }

    #[test]
    fn test_log_search_next_previous_wrap_around() {
        let mut p = ContainerPresenter::new();
        p.set_logs(make_searchable_logs());
        p.activate_log_search();
        for c in "error".chars() {
            p.push_log_search_char(c);
        }
        p.confirm_log_search();
        p.next_log_match();
        assert_eq!(p.selected_logs_line(), Some(3));
        assert_eq!(p.search_counter(), Some((2, 2)));
        p.next_log_match();
        assert_eq!(p.selected_logs_line(), Some(1));
        assert_eq!(p.search_counter(), Some((1, 2)));
        p.previous_log_match();
        assert_eq!(p.selected_logs_line(), Some(3));
        assert_eq!(p.search_counter(), Some((2, 2)));
    }

    #[test]
    fn test_log_search_no_results_keeps_selection() {
        let mut p = ContainerPresenter::new();
        p.set_logs(make_searchable_logs());
        p.next_logs_line();
        p.activate_log_search();
        for c in "zzzz".chars() {
            p.push_log_search_char(c);
        }
        p.confirm_log_search();
        assert!(p.log_search_results.is_empty());
        assert_eq!(p.search_counter(), None);
        assert_eq!(p.active_search_value(), None);
        assert_eq!(p.selected_logs_line(), Some(1));
        p.next_log_match();
        p.previous_log_match();
        assert_eq!(p.selected_logs_line(), Some(1));
    }

    #[test]
    fn test_deactivate_log_search_clears_results() {
        let mut p = ContainerPresenter::new();
        p.set_logs(make_searchable_logs());
        p.activate_log_search();
        p.push_log_search_char('e');
        p.confirm_log_search();
        assert!(!p.log_search_results.is_empty());
        p.deactivate_log_search();
        assert!(!p.log_search.is_active());
        assert!(p.log_search_results.is_empty());
        assert_eq!(p.log_search_position, 0);
    }

    #[test]
    fn test_is_log_match_and_pop_char() {
        let mut p = ContainerPresenter::new();
        p.set_logs(make_searchable_logs());
        p.activate_log_search();
        p.push_log_search_char('e');
        p.push_log_search_char('r');
        p.pop_log_search_char();
        assert_eq!(p.log_search_value(), "e");
        p.confirm_log_search();
        assert!(p.is_log_match(1));
        assert!(!p.is_log_match(0));
    }

    #[test]
    fn test_reactivating_search_resets_term() {
        let mut p = ContainerPresenter::new();
        p.set_logs(make_searchable_logs());
        p.activate_log_search();
        p.push_log_search_char('x');
        p.deactivate_log_search();
        p.activate_log_search();
        assert_eq!(p.log_search_value(), "");
    }
}
