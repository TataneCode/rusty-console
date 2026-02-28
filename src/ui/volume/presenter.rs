use crate::application::VolumeDto;
use crate::ui::common::TableSelection;

pub struct VolumePresenter {
    pub volumes: Vec<VolumeDto>,
    pub selection: TableSelection,
    pub loading: bool,
    pub error: Option<String>,
}

impl VolumePresenter {
    pub fn new() -> Self {
        VolumePresenter {
            volumes: Vec::new(),
            selection: TableSelection::new(),
            loading: false,
            error: None,
        }
    }

    pub fn set_volumes(&mut self, volumes: Vec<VolumeDto>) {
        self.volumes = volumes;
        self.selection.set_items(self.volumes.len());
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn selected_volume(&self) -> Option<&VolumeDto> {
        self.selection.selected().and_then(|i| self.volumes.get(i))
    }

    pub fn navigate_up(&mut self) {
        self.selection.previous();
    }

    pub fn navigate_down(&mut self) {
        self.selection.next();
    }
}

impl Default for VolumePresenter {
    fn default() -> Self {
        Self::new()
    }
}
