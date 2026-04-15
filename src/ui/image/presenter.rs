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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(repository: &str, tag: &str) -> ImageDto {
        ImageDto {
            id: format!("sha256:{repository}_{tag}"),
            short_id: "abc123".to_string(),
            repository: repository.to_string(),
            tag: tag.to_string(),
            full_name: format!("{repository}:{tag}"),
            size: "50 MB".to_string(),
            created: "2024-01-01".to_string(),
            in_use: false,
            is_dangling: false,
            can_delete: true,
        }
    }

    fn three_images() -> Vec<ImageDto> {
        vec![
            make_image("nginx", "latest"),
            make_image("redis", "7-alpine"),
            make_image("postgres", "16"),
        ]
    }

    #[test]
    fn test_set_images_updates_list() {
        let mut p = ImagePresenter::new();
        p.set_images(three_images());
        assert_eq!(p.images.len(), 3);
        assert_eq!(p.selection.selected(), Some(0));
        assert!(p.error.is_none());
    }

    #[test]
    fn test_filtered_images_by_repository() {
        let mut p = ImagePresenter::new();
        p.set_images(three_images());
        // Case-insensitive: "NGINX" matches "nginx"
        for c in "NGINX".chars() {
            p.push_filter_char(c);
        }
        let filtered = p.filtered_images();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].repository, "nginx");
    }

    #[test]
    fn test_filtered_images_by_tag() {
        let mut p = ImagePresenter::new();
        p.set_images(three_images());
        for c in "alpine".chars() {
            p.push_filter_char(c);
        }
        let filtered = p.filtered_images();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].repository, "redis");
    }

    #[test]
    fn test_selected_image_from_filtered_list() {
        let mut p = ImagePresenter::new();
        p.set_images(three_images());
        for c in "postgres".chars() {
            p.push_filter_char(c);
        }
        let selected = p.selected_image();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().repository, "postgres");
    }

    #[test]
    fn test_activate_deactivate_filter() {
        let mut p = ImagePresenter::new();
        p.set_images(three_images());
        p.activate_filter();
        assert!(p.filter_active);
        p.push_filter_char('z');
        p.deactivate_filter();
        assert!(!p.filter_active);
        assert!(p.filter.is_empty());
        assert_eq!(p.filtered_images().len(), 3);
    }

    #[test]
    fn test_set_details_and_clear() {
        let mut p = ImagePresenter::new();
        assert!(p.selected_image.is_none());
        p.set_details(make_image("nginx", "latest"));
        assert!(p.selected_image.is_some());
        assert_eq!(p.selected_image.as_ref().unwrap().repository, "nginx");
        p.clear_details();
        assert!(p.selected_image.is_none());
    }

    #[test]
    fn test_navigate_wraps() {
        let mut p = ImagePresenter::new();
        p.set_images(three_images());
        assert_eq!(p.selection.selected(), Some(0));
        p.navigate_up();
        assert_eq!(p.selection.selected(), Some(2));
        p.navigate_down();
        assert_eq!(p.selection.selected(), Some(0));
    }
}
