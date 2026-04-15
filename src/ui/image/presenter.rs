use crate::application::ImageDto;
use crate::ui::common::TableSelection;

pub struct ImagePresenter {
    pub images: Vec<ImageDto>,
    pub selection: TableSelection,
    pub selected_image: Option<ImageDto>,
    pub loading: bool,
    pub error: Option<String>,
    pub filter: String,
    pub filter_active: bool,
}

impl ImagePresenter {
    pub fn new() -> Self {
        ImagePresenter {
            images: Vec::new(),
            selection: TableSelection::new(),
            selected_image: None,
            loading: false,
            error: None,
            filter: String::new(),
            filter_active: false,
        }
    }

    pub fn set_images(&mut self, images: Vec<ImageDto>) {
        self.images = images;
        self.update_filtered_selection();
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn filtered_images(&self) -> Vec<&ImageDto> {
        if self.filter.is_empty() {
            self.images.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.images
                .iter()
                .filter(|i| {
                    i.repository.to_lowercase().contains(&filter_lower)
                        || i.tag.to_lowercase().contains(&filter_lower)
                })
                .collect()
        }
    }

    pub fn selected_image(&self) -> Option<&ImageDto> {
        let filtered = self.filtered_images();
        self.selection
            .selected()
            .and_then(|i| filtered.get(i).copied())
    }

    pub fn set_details(&mut self, image: ImageDto) {
        self.selected_image = Some(image);
    }

    pub fn clear_details(&mut self) {
        self.selected_image = None;
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
        let count = self.filtered_images().len();
        self.selection.set_items(count);
    }
}

impl Default for ImagePresenter {
    fn default() -> Self {
        Self::new()
    }
}
