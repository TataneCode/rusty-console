use crate::application::VolumeDto;
use crate::ui::common::TableSelection;

pub struct VolumePresenter {
    pub volumes: Vec<VolumeDto>,
    pub selection: TableSelection,
    pub loading: bool,
    pub error: Option<String>,
    pub filter: String,
    pub filter_active: bool,
}

impl VolumePresenter {
    pub fn new() -> Self {
        VolumePresenter {
            volumes: Vec::new(),
            selection: TableSelection::new(),
            loading: false,
            error: None,
            filter: String::new(),
            filter_active: false,
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
        if self.filter.is_empty() {
            self.volumes.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.volumes
                .iter()
                .filter(|v| v.name.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }

    pub fn selected_volume(&self) -> Option<&VolumeDto> {
        let filtered = self.filtered_volumes();
        self.selection
            .selected()
            .and_then(|i| filtered.get(i).copied())
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
        let count = self.filtered_volumes().len();
        self.selection.set_items(count);
    }
}

impl Default for VolumePresenter {
    fn default() -> Self {
        Self::new()
    }
}
