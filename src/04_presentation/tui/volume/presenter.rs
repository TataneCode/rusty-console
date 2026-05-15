use crate::application::volume::VolumeDto;
use crate::presentation::tui::common::{FilterState, TableSelection};

pub struct VolumePresenter {
    pub volumes: Vec<VolumeDto>,
    pub selection: TableSelection,
    pub selected_volume: Option<VolumeDto>,
    pub loading: bool,
    pub error: Option<String>,
    pub filter: FilterState,
}

pub fn filter_volumes<'a>(volumes: &'a [VolumeDto], filter: &str) -> Vec<&'a VolumeDto> {
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
            volumes: Vec::new(),
            selection: TableSelection::new(),
            selected_volume: None,
            loading: false,
            error: None,
            filter: FilterState::new(),
        }
    }

    pub fn set_volumes(&mut self, volumes: Vec<VolumeDto>) {
        self.volumes = volumes;
        self.update_filtered_selection();
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn filtered_volumes(&self) -> Vec<&VolumeDto> {
        filter_volumes(&self.volumes, self.filter.value())
    }

    pub fn selected_volume(&self) -> Option<&VolumeDto> {
        let filtered = self.filtered_volumes();
        self.selection
            .selected()
            .and_then(|i| filtered.get(i).copied())
    }

    pub fn set_details(&mut self, volume: VolumeDto) {
        self.selected_volume = Some(volume);
    }

    pub fn clear_details(&mut self) {
        self.selected_volume = None;
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
        let count = self.filtered_volumes().len();
        self.selection.set_items(count);
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
        assert_eq!(p.volumes.len(), 3);
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
}
