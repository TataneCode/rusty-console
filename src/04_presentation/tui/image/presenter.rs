use crate::application::image::{ImageDto, ImagePullProgress};
use crate::presentation::tui::common::ListCore;
use std::ops::{Deref, DerefMut};

pub struct ImagePresenter {
    pub list: ListCore<ImageDto>,
    pub selected_image: Option<ImageDto>,
    pub pull_progress: Option<ImagePullProgress>,
}

fn matches_image(image: &ImageDto, filter_lower: &str) -> bool {
    image.repository.to_lowercase().contains(filter_lower)
        || image.tag.to_lowercase().contains(filter_lower)
}

fn image_sort_key(image: &ImageDto) -> &str {
    image.full_name.as_str()
}

pub fn filter_images<'items>(images: &'items [ImageDto], filter: &str) -> Vec<&'items ImageDto> {
    if filter.is_empty() {
        images.iter().collect()
    } else {
        let filter_lower = filter.to_lowercase();
        images
            .iter()
            .filter(|image| {
                image.repository.to_lowercase().contains(&filter_lower)
                    || image.tag.to_lowercase().contains(&filter_lower)
            })
            .collect()
    }
}

impl ImagePresenter {
    pub fn new() -> Self {
        ImagePresenter {
            list: ListCore::new(matches_image, image_sort_key),
            selected_image: None,
            pull_progress: None,
        }
    }

    pub fn set_images(&mut self, images: Vec<ImageDto>) {
        self.list.set_items(images);
    }

    pub fn set_pull_progress(&mut self, progress: ImagePullProgress) {
        self.pull_progress = Some(progress);
    }

    pub fn clear_pull_progress(&mut self) {
        self.pull_progress = None;
    }

    pub fn filtered_images(&self) -> Vec<&ImageDto> {
        self.list.filtered_items()
    }

    pub fn selected_image(&self) -> Option<&ImageDto> {
        self.list.selected_item()
    }

    pub fn set_details(&mut self, image: ImageDto) {
        self.selected_image = Some(image);
    }

    pub fn clear_details(&mut self) {
        self.selected_image = None;
    }
}

impl Deref for ImagePresenter {
    type Target = ListCore<ImageDto>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for ImagePresenter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
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
        assert_eq!(p.items.len(), 3);
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
        assert!(p.filter.is_active());
        p.push_filter_char('z');
        p.deactivate_filter();
        assert!(!p.filter.is_active());
        assert!(p.filter.value().is_empty());
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

    #[test]
    fn test_set_loading_changes_state() {
        let mut p = ImagePresenter::new();
        assert!(!p.is_loading());

        p.set_loading(true);
        assert!(p.is_loading());

        p.set_loading(false);
        assert!(!p.is_loading());
    }

    #[test]
    fn test_toggle_sort_cycles_orders() {
        let mut p = ImagePresenter::new();
        p.set_images(three_images());
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_images()
            .iter()
            .map(|i| i.full_name.as_str())
            .collect();
        assert_eq!(names, vec!["nginx:latest", "postgres:16", "redis:7-alpine"]);
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_images()
            .iter()
            .map(|i| i.full_name.as_str())
            .collect();
        assert_eq!(names, vec!["redis:7-alpine", "postgres:16", "nginx:latest"]);
        p.toggle_sort();
        let names: Vec<&str> = p
            .filtered_images()
            .iter()
            .map(|i| i.full_name.as_str())
            .collect();
        assert_eq!(names, vec!["nginx:latest", "redis:7-alpine", "postgres:16"]);
    }

    #[test]
    fn test_loading_state_defaults_to_false() {
        let p = ImagePresenter::new();
        assert!(!p.is_loading());
        assert!(!p.loading);
    }
}
