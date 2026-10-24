use crate::application::container::ContainerStatsUpdate;
use crate::application::stack::{StackContainerDto, StackDto};
use crate::presentation::tui::common::ListCore;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

pub struct StackPresenter {
    pub list: ListCore<StackDto>,
    pub containers: ListCore<StackContainerDto>,
    stats_by_container_id: HashMap<String, crate::application::container::ContainerRuntimeStatsDto>,
}

fn matches_stack(stack: &StackDto, filter_lower: &str) -> bool {
    stack.name.to_lowercase().contains(filter_lower)
}

fn stack_sort_key(stack: &StackDto) -> &str {
    stack.name.as_str()
}

fn matches_stack_container(container: &StackContainerDto, filter_lower: &str) -> bool {
    container.name.to_lowercase().contains(filter_lower)
        || container.image.to_lowercase().contains(filter_lower)
}

fn stack_container_sort_key(container: &StackContainerDto) -> &str {
    container.name.as_str()
}

impl StackPresenter {
    pub fn new() -> Self {
        StackPresenter {
            list: ListCore::new(matches_stack, stack_sort_key),
            containers: ListCore::new(matches_stack_container, stack_container_sort_key),
            stats_by_container_id: HashMap::new(),
        }
    }

    pub fn set_stacks(&mut self, stacks: Vec<StackDto>) {
        self.list.set_items(stacks);
    }

    pub fn filtered_stacks(&self) -> Vec<&StackDto> {
        self.list.filtered_items()
    }

    pub fn selected_stack(&self) -> Option<&StackDto> {
        self.list.selected_item()
    }

    pub fn select_stack_by_name(&mut self, stack_name: &str) -> bool {
        let selected = self
            .filtered_stacks()
            .iter()
            .position(|stack| stack.name == stack_name);

        self.selection.select(selected);
        selected.is_some()
    }

    pub fn set_stack_containers(&mut self, containers: Vec<StackContainerDto>) {
        self.containers.set_items(containers);
        self.reapply_runtime_stats();
    }

    pub fn filtered_stack_containers(&self) -> Vec<&StackContainerDto> {
        self.containers.filtered_items()
    }

    pub fn selected_stack_container(&self) -> Option<&StackContainerDto> {
        self.containers.selected_item()
    }

    pub fn navigate_container_up(&mut self) {
        self.containers.navigate_up();
    }

    pub fn navigate_container_down(&mut self) {
        self.containers.navigate_down();
    }

    pub fn apply_stats_update(&mut self, update: ContainerStatsUpdate) {
        self.stats_by_container_id
            .insert(update.container_id.clone(), update.stats.clone());

        if let Some(container) = self
            .containers
            .items
            .iter_mut()
            .find(|container| container.id == update.container_id)
        {
            container.runtime_stats = Some(update.stats);
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
        for container in &mut self.containers.items {
            container.runtime_stats = self.stats_by_container_id.get(&container.id).cloned();
        }
    }
}

impl Deref for StackPresenter {
    type Target = ListCore<StackDto>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for StackPresenter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl Default for StackPresenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::stack::StackContainerState;

    fn make_stack(name: &str, total: usize) -> StackDto {
        StackDto {
            name: name.to_string(),
            container_count: total,
            running_count: 0,
            containers: vec![],
        }
    }

    fn three_stacks() -> Vec<StackDto> {
        vec![
            make_stack("app-frontend", 2),
            make_stack("app-backend", 3),
            make_stack("db-postgres", 1),
        ]
    }

    #[test]
    fn test_new_empty() {
        let p = StackPresenter::new();
        assert!(p.items.is_empty());
        assert!(p.selected_stack().is_none());
        assert!(p.containers.items.is_empty());
        assert!(p.selected_stack_container().is_none());
        assert!(!p.loading);
        assert!(!p.containers.loading);
        assert!(p.error.is_none());
        assert!(p.filter.value().is_empty());
        assert!(!p.filter.is_active());
        assert!(p.containers.filter.value().is_empty());
        assert!(!p.containers.filter.is_active());
    }

    #[test]
    fn test_set_stacks_selects_first() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        assert_eq!(p.items.len(), 2);
        assert_eq!(p.selected_stack().unwrap().name, "app-a");
    }

    #[test]
    fn test_set_stacks_clears_error() {
        let mut p = StackPresenter::new();
        p.set_error("something failed".to_string());
        assert!(p.error.is_some());
        p.set_stacks(three_stacks());
        assert!(p.error.is_none());
    }

    #[test]
    fn test_navigate_down() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        p.navigate_down();
        assert_eq!(p.selected_stack().unwrap().name, "app-b");
    }

    #[test]
    fn test_navigate_wraps_around() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        p.navigate_down();
        p.navigate_down(); // wraps to 0
        assert_eq!(p.selected_stack().unwrap().name, "app-a");
    }

    #[test]
    fn test_navigate_up_from_start_wraps() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        p.navigate_up(); // wraps to last
        assert_eq!(p.selected_stack().unwrap().name, "app-b");
    }

    #[test]
    fn test_selected_stack_empty() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![]);
        assert!(p.selected_stack().is_none());
    }

    #[test]
    fn test_set_stacks_replaces_previous() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![make_stack("old", 1)]);
        p.set_stacks(vec![make_stack("new-a", 2), make_stack("new-b", 3)]);
        assert_eq!(p.items.len(), 2);
        assert_eq!(p.selected_stack().unwrap().name, "new-a");
    }

    #[test]
    fn test_filtered_stacks_no_filter() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        assert_eq!(p.filtered_stacks().len(), 3);
    }

    #[test]
    fn test_filtered_stacks_by_name() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        for c in "app".chars() {
            p.push_filter_char(c);
        }
        let filtered = p.filtered_stacks();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|s| s.name.contains("app")));
    }

    #[test]
    fn test_filtered_stacks_case_insensitive() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        for c in "DB".chars() {
            p.push_filter_char(c);
        }
        let filtered = p.filtered_stacks();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "db-postgres");
    }

    #[test]
    fn test_filtered_stacks_no_match() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        for c in "zzzzz".chars() {
            p.push_filter_char(c);
        }
        assert!(p.filtered_stacks().is_empty());
    }

    #[test]
    fn test_selected_stack_from_filtered_list() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        for c in "postgres".chars() {
            p.push_filter_char(c);
        }
        let selected = p.selected_stack();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "db-postgres");
    }

    #[test]
    fn test_activate_deactivate_filter() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        p.activate_filter();
        assert!(p.filter.is_active());
        p.push_filter_char('x');
        p.deactivate_filter();
        assert!(!p.filter.is_active());
        assert!(p.filter.value().is_empty());
        assert_eq!(p.filtered_stacks().len(), 3);
    }

    #[test]
    fn test_filter_narrows_selection() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        p.navigate_down();
        p.navigate_down();
        assert_eq!(p.selection.selected(), Some(2));
        for c in "db".chars() {
            p.push_filter_char(c);
        }
        assert_eq!(p.filtered_stacks().len(), 1);
        assert_eq!(p.selection.selected(), Some(0));
    }

    #[test]
    fn test_select_stack_by_name() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        assert!(p.select_stack_by_name("db-postgres"));
        assert_eq!(p.selected_stack().unwrap().name, "db-postgres");
    }

    fn make_stack_container(name: &str, image: &str) -> StackContainerDto {
        StackContainerDto {
            id: format!("id_{name}"),
            name: name.to_string(),
            image: image.to_string(),
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

    fn three_stack_containers() -> Vec<StackContainerDto> {
        vec![
            make_stack_container("alpha", "nginx:latest"),
            make_stack_container("beta", "redis:7"),
            make_stack_container("gamma", "nginx:alpine"),
        ]
    }

    #[test]
    fn test_set_stack_containers_selects_first() {
        let mut p = StackPresenter::new();
        p.set_stack_containers(three_stack_containers());
        assert_eq!(p.selected_stack_container().unwrap().name, "alpha");
        p.navigate_container_down();
        assert_eq!(p.selected_stack_container().unwrap().name, "beta");
    }

    #[test]
    fn test_stack_containers_filter_by_name_and_image() {
        let mut p = StackPresenter::new();
        p.set_stack_containers(three_stack_containers());
        assert_eq!(p.filtered_stack_containers().len(), 3);

        p.containers.push_filter_char('a');
        p.containers.push_filter_char('l');
        p.containers.push_filter_char('p');
        p.containers.push_filter_char('h');
        let filtered = p.filtered_stack_containers();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "alpha");

        let mut p = StackPresenter::new();
        p.set_stack_containers(three_stack_containers());
        for c in "NGINX".chars() {
            p.containers.push_filter_char(c);
        }
        assert_eq!(p.filtered_stack_containers().len(), 2);
    }

    #[test]
    fn test_stack_containers_filter_keeps_selection_in_bounds() {
        let mut p = StackPresenter::new();
        p.set_stack_containers(three_stack_containers());
        p.navigate_container_down();
        p.navigate_container_down();
        assert_eq!(p.containers.selection.selected(), Some(2));
        for c in "beta".chars() {
            p.containers.push_filter_char(c);
        }
        assert_eq!(p.filtered_stack_containers().len(), 1);
        assert_eq!(p.selected_stack_container().unwrap().name, "beta");
    }

    #[test]
    fn test_stack_containers_toggle_sort_cycles_orders() {
        let mut p = StackPresenter::new();
        p.set_stack_containers(three_stack_containers());
        p.containers.toggle_sort();
        let names: Vec<&str> = p
            .filtered_stack_containers()
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
        p.containers.toggle_sort();
        let names: Vec<&str> = p
            .filtered_stack_containers()
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(names, vec!["gamma", "beta", "alpha"]);
    }

    #[test]
    fn test_stack_container_runtime_stats() {
        let mut p = StackPresenter::new();
        p.set_stack_containers(three_stack_containers());
        assert_eq!(p.filtered_stack_containers()[0].cpu_display(), "N/A");

        p.apply_stats_update(crate::application::container::ContainerStatsUpdate {
            container_id: "id_alpha".to_string(),
            stats: crate::application::container::ContainerRuntimeStatsDto {
                cpu_percent: 12.5,
                memory_usage: crate::domain::shared::ByteSize::new(512),
                memory_limit: crate::domain::shared::ByteSize::new(1024),
                memory_percent: 50.0,
                network_rx: crate::domain::shared::ByteSize::new(256),
                network_tx: crate::domain::shared::ByteSize::new(128),
            },
        });
        assert_eq!(p.filtered_stack_containers()[0].cpu_display(), "12.5%");
        assert_eq!(
            p.filtered_stack_containers()[0].memory_display(),
            "512 B (50%)"
        );

        p.retain_runtime_stats(&HashSet::from(["id_beta".to_string()]));
        assert_eq!(p.filtered_stack_containers()[0].cpu_display(), "N/A");

        p.apply_stats_update(crate::application::container::ContainerStatsUpdate {
            container_id: "id_beta".to_string(),
            stats: crate::application::container::ContainerRuntimeStatsDto {
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
            .filtered_stack_containers()
            .iter()
            .all(|container| container.runtime_stats.is_none()));
    }

    #[test]
    fn test_stack_container_loading_state() {
        let mut p = StackPresenter::new();
        assert!(!p.containers.is_loading());
        p.containers.set_loading(true);
        assert!(p.containers.is_loading());
        p.containers.set_loading(false);
        assert!(!p.containers.is_loading());
    }

    #[test]
    fn test_set_loading_changes_state() {
        let mut p = StackPresenter::new();
        assert!(!p.is_loading());
        p.set_loading(true);
        assert!(p.is_loading());
        p.set_loading(false);
        assert!(!p.is_loading());
    }

    #[test]
    fn test_toggle_sort_cycles_orders() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_stacks()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(names, vec!["app-backend", "app-frontend", "db-postgres"]);
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_stacks()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(names, vec!["db-postgres", "app-frontend", "app-backend"]);
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_stacks()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert_eq!(names, vec!["app-frontend", "app-backend", "db-postgres"]);
    }

    #[test]
    fn test_loading_state_defaults_to_false() {
        let p = StackPresenter::new();
        assert!(!p.is_loading());
        assert!(!p.loading);
    }
}
