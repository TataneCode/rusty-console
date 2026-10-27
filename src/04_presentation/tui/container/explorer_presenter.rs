//! Presenter state for the container file explorer screen: current directory
//! listing, navigation history (for `h` = parent), and the loaded file view.
//!
//! Selection, filtering (`/`) and sorting (`o`, 3 states) are delegated to
//! the shared [`ListCore`].

use crate::application::container::{FileContentDto, FileEntryDto, FileListingDto};
use crate::presentation::tui::common::ListCore;
use std::ops::{Deref, DerefMut};

pub struct ContainerExplorerPresenter {
    pub list: ListCore<FileEntryDto>,
    pub listing: Option<FileListingDto>,
    pub container_id: String,
    pub container_name: String,
    /// Directories visited, oldest first; the last entry is the current path.
    path_stack: Vec<String>,
    pub file_view: Option<FileContentDto>,
    pub file_view_scroll: usize,
    pub file_view_path: Option<String>,
}

fn matches_file_entry(entry: &FileEntryDto, filter_lower: &str) -> bool {
    entry.name.to_lowercase().contains(filter_lower)
}

fn file_sort_key(entry: &FileEntryDto) -> &str {
    entry.name.as_str()
}

impl ContainerExplorerPresenter {
    pub fn new() -> Self {
        ContainerExplorerPresenter {
            list: ListCore::new(matches_file_entry, file_sort_key),
            listing: None,
            container_id: String::new(),
            container_name: String::new(),
            path_stack: Vec::new(),
            file_view: None,
            file_view_scroll: 0,
            file_view_path: None,
        }
    }

    /// Opens the explorer on the root directory of the container.
    pub fn open(&mut self, container_id: &str, container_name: &str) {
        self.container_id = container_id.to_string();
        self.container_name = container_name.to_string();
        self.path_stack = vec!["/".to_string()];
        self.listing = None;
        self.file_view = None;
        self.file_view_path = None;
        self.file_view_scroll = 0;
        self.list.set_items(Vec::new());
    }

    pub fn current_path(&self) -> String {
        self.path_stack
            .last()
            .cloned()
            .unwrap_or_else(|| "/".to_string())
    }

    pub fn can_go_up(&self) -> bool {
        self.path_stack.len() > 1
    }

    /// Path of the directory entry under the selection, if it is a directory.
    pub fn selected_entry(&self) -> Option<&FileEntryDto> {
        self.list.selected_item()
    }

    /// Path of the selected directory, when the selection is a directory.
    pub fn selected_dir_path(&self) -> Option<String> {
        let entry = self.selected_entry()?;
        entry.is_dir().then(|| entry.path.clone())
    }

    /// Sets the listing for a directory entered by navigation: pushes the path
    /// on the stack unless it is already the current one (initial load).
    pub fn set_listing(&mut self, listing: FileListingDto) {
        if self.path_stack.last() != Some(&listing.path) {
            self.path_stack.push(listing.path.clone());
        }
        self.listing = Some(listing);
        self.list.set_items(
            self.listing
                .as_ref()
                .map(|listing| listing.entries.clone())
                .unwrap_or_default(),
        );
    }

    /// Replaces the current directory with the given listing (refresh).
    pub fn refresh_listing(&mut self, listing: FileListingDto) {
        if let Some(last) = self.path_stack.last_mut() {
            *last = listing.path.clone();
        }
        self.listing = Some(listing);
        self.list.set_items(
            self.listing
                .as_ref()
                .map(|listing| listing.entries.clone())
                .unwrap_or_default(),
        );
    }

    /// Navigates back to the parent directory of the current listing.
    pub fn go_to_parent(&mut self) {
        if self.path_stack.len() > 1 {
            self.path_stack.pop();
        }
        self.listing = None;
        self.list.set_items(Vec::new());
    }

    pub fn clear(&mut self) {
        self.listing = None;
        self.container_id.clear();
        self.container_name.clear();
        self.path_stack.clear();
        self.file_view = None;
        self.file_view_path = None;
        self.file_view_scroll = 0;
        self.list.set_items(Vec::new());
    }

    pub fn show_file(&mut self, path: &str, content: FileContentDto) {
        self.file_view = Some(content);
        self.file_view_path = Some(path.to_string());
        self.file_view_scroll = 0;
    }

    pub fn close_file_view(&mut self) {
        self.file_view = None;
        self.file_view_path = None;
        self.file_view_scroll = 0;
    }

    pub fn file_view_line_count(&self) -> usize {
        self.file_view
            .as_ref()
            .and_then(|content| content.text())
            .map(|text| text.lines().count())
            .unwrap_or(0)
    }

    pub fn scroll_file_view_down(&mut self, viewport_height: usize) {
        let max_scroll = self.file_view_line_count().saturating_sub(viewport_height);
        if self.file_view_scroll < max_scroll {
            self.file_view_scroll += 1;
        }
    }

    pub fn scroll_file_view_up(&mut self) {
        self.file_view_scroll = self.file_view_scroll.saturating_sub(1);
    }
}

impl Deref for ContainerExplorerPresenter {
    type Target = ListCore<FileEntryDto>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for ContainerExplorerPresenter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl Default for ContainerExplorerPresenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::container::FileKind;
    use chrono::{TimeZone, Utc};

    fn make_entry(name: &str, kind: FileKind, path: &str) -> FileEntryDto {
        FileEntryDto {
            name: name.to_string(),
            path: path.to_string(),
            kind,
            size: 10,
            modified: Some(Utc.with_ymd_and_hms(2026, 10, 26, 10, 30, 0).unwrap()),
            permissions: "-rw-r--r--".to_string(),
            link_target: None,
        }
    }

    fn make_listing(path: &str, entries: Vec<FileEntryDto>) -> FileListingDto {
        FileListingDto {
            container_id: "abc123".to_string(),
            container_name: "web".to_string(),
            path: path.to_string(),
            entries,
        }
    }

    fn child_path(dir: &str, name: &str) -> String {
        if dir == "/" {
            format!("/{name}")
        } else {
            format!("{dir}/{name}")
        }
    }

    fn sample_listing(path: &str) -> FileListingDto {
        make_listing(
            path,
            vec![
                make_entry("etc", FileKind::Directory, &child_path(path, "etc")),
                make_entry("app.conf", FileKind::File, &child_path(path, "app.conf")),
            ],
        )
    }

    #[test]
    fn test_open_resets_state_to_root() {
        let mut p = ContainerExplorerPresenter::new();
        p.open("abc123", "web");
        assert_eq!(p.container_id, "abc123");
        assert_eq!(p.container_name, "web");
        assert_eq!(p.current_path(), "/");
        assert!(!p.can_go_up());
        assert!(p.listing.is_none());
    }

    #[test]
    fn test_set_listing_pushes_path_and_populates_items() {
        let mut p = ContainerExplorerPresenter::new();
        p.open("abc123", "web");
        p.set_listing(sample_listing("/"));
        assert_eq!(p.current_path(), "/");
        assert_eq!(p.list.items.len(), 2);
        assert_eq!(p.selection.selected(), Some(0));
    }

    #[test]
    fn test_set_listing_entering_subdirectory_enables_go_up() {
        let mut p = ContainerExplorerPresenter::new();
        p.open("abc123", "web");
        p.set_listing(sample_listing("/"));
        let dir = p.selected_dir_path().unwrap();
        assert_eq!(dir, "/etc");
        p.set_listing(sample_listing("/etc"));
        assert_eq!(p.current_path(), "/etc");
        assert!(p.can_go_up());
    }

    #[test]
    fn test_selected_dir_path_is_none_for_files() {
        let mut p = ContainerExplorerPresenter::new();
        p.open("abc123", "web");
        p.set_listing(sample_listing("/"));
        p.navigate_down();
        assert!(p.selected_dir_path().is_none());
        assert_eq!(p.selected_entry().unwrap().name, "app.conf");
    }

    #[test]
    fn test_go_to_parent_pops_path_and_clears_listing() {
        let mut p = ContainerExplorerPresenter::new();
        p.open("abc123", "web");
        p.set_listing(sample_listing("/"));
        p.set_listing(sample_listing("/etc"));
        p.go_to_parent();
        assert_eq!(p.current_path(), "/");
        assert!(!p.can_go_up());
        assert!(p.listing.is_none());
        assert!(p.list.items.is_empty());
    }

    #[test]
    fn test_go_to_parent_at_root_is_noop() {
        let mut p = ContainerExplorerPresenter::new();
        p.open("abc123", "web");
        p.set_listing(sample_listing("/"));
        p.go_to_parent();
        assert_eq!(p.current_path(), "/");
    }

    #[test]
    fn test_refresh_listing_keeps_current_path() {
        let mut p = ContainerExplorerPresenter::new();
        p.open("abc123", "web");
        p.set_listing(sample_listing("/"));
        p.set_listing(sample_listing("/etc"));
        p.refresh_listing(sample_listing("/etc"));
        assert_eq!(p.current_path(), "/etc");
        assert_eq!(p.list.items.len(), 2);
    }

    #[test]
    fn test_filter_narrows_entries_by_name() {
        let mut p = ContainerExplorerPresenter::new();
        p.open("abc123", "web");
        p.set_listing(sample_listing("/"));
        p.push_filter_char('a');
        p.push_filter_char('p');
        p.push_filter_char('p');
        let filtered = p.filtered_items();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "app.conf");
    }

    #[test]
    fn test_toggle_sort_cycles_three_states() {
        let mut p = ContainerExplorerPresenter::new();
        p.open("abc123", "web");
        p.set_listing(make_listing(
            "/",
            vec![
                make_entry("zeta", FileKind::File, "/zeta"),
                make_entry("alpha", FileKind::File, "/alpha"),
            ],
        ));
        fn names(p: &ContainerExplorerPresenter) -> Vec<String> {
            p.filtered_items().iter().map(|e| e.name.clone()).collect()
        }
        assert_eq!(names(&p), vec!["zeta", "alpha"]);
        p.toggle_sort();
        assert_eq!(names(&p), vec!["alpha", "zeta"]);
        p.toggle_sort();
        assert_eq!(names(&p), vec!["zeta", "alpha"]);
        p.toggle_sort();
        assert_eq!(names(&p), vec!["zeta", "alpha"]);
    }

    #[test]
    fn test_file_view_scroll_bounds() {
        let mut p = ContainerExplorerPresenter::new();
        let content = FileContentDto {
            data: (0..30)
                .map(|i| format!("line{i}\n"))
                .collect::<String>()
                .into_bytes(),
            total_size: 30 * 6,
            truncated: false,
        };
        p.show_file("/f.txt", content);
        assert_eq!(p.file_view_line_count(), 30);
        assert_eq!(p.file_view_scroll, 0);
        p.scroll_file_view_up();
        assert_eq!(p.file_view_scroll, 0);
        for _ in 0..100 {
            p.scroll_file_view_down(10);
        }
        assert_eq!(p.file_view_scroll, 20);
        p.close_file_view();
        assert!(p.file_view.is_none());
        assert_eq!(p.file_view_scroll, 0);
    }

    #[test]
    fn test_file_view_line_count_zero_for_binary_content() {
        let mut p = ContainerExplorerPresenter::new();
        p.show_file(
            "/bin/app",
            FileContentDto {
                data: vec![0, 1, 2],
                total_size: 3,
                truncated: false,
            },
        );
        assert_eq!(p.file_view_line_count(), 0);
    }

    #[test]
    fn test_clear_resets_everything() {
        let mut p = ContainerExplorerPresenter::new();
        p.open("abc123", "web");
        p.set_listing(sample_listing("/"));
        p.clear();
        assert!(p.listing.is_none());
        assert!(p.container_id.is_empty());
        assert!(p.path_stack.is_empty());
        assert_eq!(p.current_path(), "/");
    }
}
