use crate::application::volume::VolumeDto;
use crate::presentation::tui::common::ListCore;
use std::ops::{Deref, DerefMut};

pub struct VolumePresenter {
    pub list: ListCore<VolumeDto>,
    pub selected_volume: Option<VolumeDto>,
}

fn matches_volume(volume: &VolumeDto, filter_lower: &str) -> bool {
    volume.name.to_lowercase().contains(filter_lower)
}

fn volume_sort_key(volume: &VolumeDto) -> &str {
    volume.name.as_str()
}

pub fn filter_volumes<'items>(
    volumes: &'items [VolumeDto],
    filter: &str,
) -> Vec<&'items VolumeDto> {
    if filter.is_empty() {
        volumes.iter().collect()
    } else {
        let filter_lower = filter.to_lowercase();
        volumes
            .iter()
            .filter(|volume| volume.name.to_lowercase().contains(&filter_lower))
            .collect()
    }
}

impl VolumePresenter {
    pub fn new() -> Self {
        VolumePresenter {
            list: ListCore::new(matches_volume, volume_sort_key),
            selected_volume: None,
        }
    }

    pub fn set_volumes(&mut self, volumes: Vec<VolumeDto>) {
        self.list.set_items(volumes);
    }

    pub fn filtered_volumes(&self) -> Vec<&VolumeDto> {
        self.list.filtered_items()
    }

    pub fn selected_volume(&self) -> Option<&VolumeDto> {
        self.list.selected_item()
    }

    pub fn set_details(&mut self, volume: VolumeDto) {
        self.selected_volume = Some(volume);
    }

    pub fn clear_details(&mut self) {
        self.selected_volume = None;
    }
}

impl Deref for VolumePresenter {
    type Target = ListCore<VolumeDto>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for VolumePresenter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl Default for VolumePresenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_volume(name: &str) -> VolumeDto {
        VolumeDto {
            id: format!("id_{name}"),
            name: name.to_string(),
            driver: "local".to_string(),
            mountpoint: format!("/var/lib/docker/volumes/{name}/_data"),
            size: "10 MB".to_string(),
            created: "2024-01-01".to_string(),
            in_use: false,
            linked_containers: Vec::new(),
            can_delete: true,
        }
    }

    fn three_volumes() -> Vec<VolumeDto> {
        vec![
            make_volume("pgdata"),
            make_volume("redis-cache"),
            make_volume("app-logs"),
        ]
    }

    #[test]
    fn test_set_volumes_updates_list() {
        let mut p = VolumePresenter::new();
        p.set_volumes(three_volumes());
        assert_eq!(p.items.len(), 3);
        assert_eq!(p.selection.selected(), Some(0));
        assert!(p.error.is_none());
    }

    #[test]
    fn test_filtered_volumes_by_name() {
        let mut p = VolumePresenter::new();
        p.set_volumes(three_volumes());
        // Case-insensitive: "REDIS" should match "redis-cache"
        for c in "REDIS".chars() {
            p.push_filter_char(c);
        }
        let filtered = p.filtered_volumes();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "redis-cache");
    }

    #[test]
    fn test_selected_volume_from_filtered_list() {
        let mut p = VolumePresenter::new();
        p.set_volumes(three_volumes());
        for c in "app".chars() {
            p.push_filter_char(c);
        }
        let selected = p.selected_volume();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "app-logs");
    }

    #[test]
    fn test_set_details_and_clear() {
        let mut p = VolumePresenter::new();
        let volume = make_volume("db-data");

        p.set_details(volume.clone());
        assert_eq!(
            p.selected_volume.as_ref().map(|v| v.name.as_str()),
            Some("db-data")
        );

        p.clear_details();
        assert!(p.selected_volume.is_none());
    }

    #[test]
    fn test_activate_deactivate_filter() {
        let mut p = VolumePresenter::new();
        p.set_volumes(three_volumes());
        p.activate_filter();
        assert!(p.filter.is_active());
        p.push_filter_char('x');
        p.deactivate_filter();
        assert!(!p.filter.is_active());
        assert!(p.filter.value().is_empty());
        // All items visible again
        assert_eq!(p.filtered_volumes().len(), 3);
    }

    #[test]
    fn test_filter_narrows_selection() {
        let mut p = VolumePresenter::new();
        p.set_volumes(three_volumes());
        p.navigate_down();
        p.navigate_down();
        assert_eq!(p.selection.selected(), Some(2));
        // Filter to 1 item
        for c in "pgdata".chars() {
            p.push_filter_char(c);
        }
        assert_eq!(p.filtered_volumes().len(), 1);
        assert_eq!(p.selection.selected(), Some(0));
    }

    #[test]
    fn test_navigate_wraps() {
        let mut p = VolumePresenter::new();
        p.set_volumes(three_volumes());
        assert_eq!(p.selection.selected(), Some(0));
        // Wrap backwards from 0
        p.navigate_up();
        assert_eq!(p.selection.selected(), Some(2));
        // Wrap forwards from last
        p.navigate_down();
        assert_eq!(p.selection.selected(), Some(0));
    }

    #[test]
    fn test_set_loading_changes_state() {
        let mut p = VolumePresenter::new();
        assert!(!p.is_loading());

        p.set_loading(true);
        assert!(p.is_loading());

        p.set_loading(false);
        assert!(!p.is_loading());
    }

    #[test]
    fn test_toggle_sort_cycles_orders() {
        let mut p = VolumePresenter::new();
        p.set_volumes(three_volumes());
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_volumes()
            .iter()
            .map(|v| v.name.as_str())
            .collect();
        assert_eq!(names, vec!["app-logs", "pgdata", "redis-cache"]);
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_volumes()
            .iter()
            .map(|v| v.name.as_str())
            .collect();
        assert_eq!(names, vec!["redis-cache", "pgdata", "app-logs"]);
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_volumes()
            .iter()
            .map(|v| v.name.as_str())
            .collect();
        assert_eq!(names, vec!["pgdata", "redis-cache", "app-logs"]);
    }

    #[test]
    fn test_sort_combines_with_filter() {
        let mut p = VolumePresenter::new();
        p.set_volumes(three_volumes());
        for c in "e".chars() {
            p.push_filter_char(c);
        }
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_volumes()
            .iter()
            .map(|v| v.name.as_str())
            .collect();
        assert_eq!(names, vec!["redis-cache"]);
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_volumes()
            .iter()
            .map(|v| v.name.as_str())
            .collect();
        assert_eq!(names, vec!["redis-cache"]);
    }

    #[test]
    fn test_loading_state_defaults_to_false() {
        let p = VolumePresenter::new();
        assert!(!p.is_loading());
        assert!(!p.loading);
    }
}
