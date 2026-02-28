use crate::application::ImageDto;
use crate::ui::common::TableSelection;

pub struct ImagePresenter {
    pub images: Vec<ImageDto>,
    pub selection: TableSelection,
    pub selected_image: Option<ImageDto>,
    pub loading: bool,
    pub error: Option<String>,
}

impl ImagePresenter {
    pub fn new() -> Self {
        ImagePresenter {
            images: Vec::new(),
            selection: TableSelection::new(),
            selected_image: None,
            loading: false,
            error: None,
        }
    }

    pub fn set_images(&mut self, images: Vec<ImageDto>) {
        self.images = images;
        self.selection.set_items(self.images.len());
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn selected_image(&self) -> Option<&ImageDto> {
        self.selection.selected().and_then(|i| self.images.get(i))
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
}

impl Default for ImagePresenter {
    fn default() -> Self {
        Self::new()
    }
}
