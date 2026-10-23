//! Shared state for list screens: items, selection, filter, sort, loading, error.
//!
//! [`ListCore`] factors out the identical list behavior duplicated by the five
//! domain presenters (container, image, network, volume, stack). Each presenter
//! embeds it and exposes it through `Deref`/`DerefMut`, so call sites keep using
//! `presenter.selection`, `presenter.filter`, `presenter.navigate_up()`, etc.
//!
//! The two variable parts are injected at construction:
//! - `matches`: case-insensitive substring predicate against the filter text
//! - `sort_key`: the name used for `o` (toggle sort)

use super::{apply_name_sort, FilterState, SortOrder, TableSelection};

pub struct ListCore<TItem> {
    pub items: Vec<TItem>,
    pub selection: TableSelection,
    pub loading: bool,
    pub error: Option<String>,
    pub filter: FilterState,
    pub sort_order: SortOrder,
    matches: fn(&TItem, &str) -> bool,
    sort_key: fn(&TItem) -> &str,
}

impl<TItem> ListCore<TItem> {
    pub fn new(matches: fn(&TItem, &str) -> bool, sort_key: fn(&TItem) -> &str) -> Self {
        ListCore {
            items: Vec::new(),
            selection: TableSelection::new(),
            loading: false,
            error: None,
            filter: FilterState::new(),
            sort_order: SortOrder::default(),
            matches,
            sort_key,
        }
    }

    pub fn set_items(&mut self, items: Vec<TItem>) {
        self.items = items;
        self.update_filtered_selection();
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn filtered_items(&self) -> Vec<&TItem> {
        let filter = self.filter.value();
        let mut filtered: Vec<&TItem> = if filter.is_empty() {
            self.items.iter().collect()
        } else {
            let filter_lower = filter.to_lowercase();
            self.items
                .iter()
                .filter(|item| (self.matches)(item, &filter_lower))
                .collect()
        };
        apply_name_sort(&mut filtered, self.sort_order, self.sort_key);
        filtered
    }

    pub fn toggle_sort(&mut self) {
        self.sort_order = self.sort_order.next();
        self.update_filtered_selection();
    }

    pub fn selected_item(&self) -> Option<&TItem> {
        let filtered = self.filtered_items();
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
        let count = self.filtered_items().len();
        self.selection.set_items(count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Row {
        name: String,
        tag: String,
    }

    fn matches_name(row: &Row, filter_lower: &str) -> bool {
        row.name.to_lowercase().contains(filter_lower)
            || row.tag.to_lowercase().contains(filter_lower)
    }

    fn sort_key(row: &Row) -> &str {
        row.name.as_str()
    }

    fn make_row(name: &str, tag: &str) -> Row {
        Row {
            name: name.to_string(),
            tag: tag.to_string(),
        }
    }

    fn three_rows() -> Vec<Row> {
        vec![
            make_row("alpha", "one"),
            make_row("beta", "two"),
            make_row("gamma", "three"),
        ]
    }

    fn new_core() -> ListCore<Row> {
        ListCore::new(matches_name, sort_key)
    }

    #[test]
    fn test_new_creates_empty_state() {
        let core = new_core();
        assert!(core.items.is_empty());
        assert!(!core.loading);
        assert!(core.error.is_none());
        assert!(core.filter.value().is_empty());
        assert!(!core.filter.is_active());
        assert!(core.selection.selected().is_none());
    }

    #[test]
    fn test_set_items_updates_list_and_selection() {
        let mut core = new_core();
        core.set_items(three_rows());
        assert_eq!(core.items.len(), 3);
        assert_eq!(core.selection.selected(), Some(0));
    }

    #[test]
    fn test_set_items_clears_error() {
        let mut core = new_core();
        core.set_error("something failed".to_string());
        assert!(core.error.is_some());
        core.set_items(three_rows());
        assert!(core.error.is_none());
    }

    #[test]
    fn test_filtered_items_no_filter() {
        let mut core = new_core();
        core.set_items(three_rows());
        assert_eq!(core.filtered_items().len(), 3);
    }

    #[test]
    fn test_filtered_items_case_insensitive_on_both_fields() {
        let mut core = new_core();
        core.set_items(three_rows());
        core.push_filter_char('A');
        core.push_filter_char('L');
        core.push_filter_char('P');
        core.push_filter_char('H');
        let filtered = core.filtered_items();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "alpha");

        let mut core = new_core();
        core.set_items(three_rows());
        for c in "TWO".chars() {
            core.push_filter_char(c);
        }
        let filtered = core.filtered_items();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "beta");
    }

    #[test]
    fn test_filtered_items_no_match() {
        let mut core = new_core();
        core.set_items(three_rows());
        for c in "zzzzz".chars() {
            core.push_filter_char(c);
        }
        assert!(core.filtered_items().is_empty());
    }

    #[test]
    fn test_selected_item_from_filtered_list() {
        let mut core = new_core();
        core.set_items(three_rows());
        for c in "beta".chars() {
            core.push_filter_char(c);
        }
        let selected = core.selected_item();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "beta");
    }

    #[test]
    fn test_activate_deactivate_filter() {
        let mut core = new_core();
        core.set_items(three_rows());
        core.activate_filter();
        assert!(core.filter.is_active());
        core.push_filter_char('a');
        assert_eq!(core.filter.value(), "a");
        core.deactivate_filter();
        assert!(!core.filter.is_active());
        assert!(core.filter.value().is_empty());
        assert_eq!(core.filtered_items().len(), 3);
    }

    #[test]
    fn test_push_pop_filter_char() {
        let mut core = new_core();
        core.set_items(three_rows());
        core.push_filter_char('a');
        core.push_filter_char('b');
        assert_eq!(core.filter.value(), "ab");
        core.pop_filter_char();
        assert_eq!(core.filter.value(), "a");
        core.pop_filter_char();
        assert!(core.filter.value().is_empty());
    }

    #[test]
    fn test_filter_updates_selection_bounds() {
        let mut core = new_core();
        core.set_items(three_rows());
        core.navigate_down();
        core.navigate_down();
        assert_eq!(core.selection.selected(), Some(2));
        for c in "beta".chars() {
            core.push_filter_char(c);
        }
        assert_eq!(core.filtered_items().len(), 1);
        assert_eq!(core.selection.selected(), Some(0));
    }

    #[test]
    fn test_navigate_up_down_wraps() {
        let mut core = new_core();
        core.set_items(three_rows());
        assert_eq!(core.selection.selected(), Some(0));
        core.navigate_down();
        assert_eq!(core.selection.selected(), Some(1));
        core.navigate_down();
        assert_eq!(core.selection.selected(), Some(2));
        core.navigate_down();
        assert_eq!(core.selection.selected(), Some(0));
        core.navigate_up();
        assert_eq!(core.selection.selected(), Some(2));
    }

    #[test]
    fn test_set_loading_changes_state() {
        let mut core = new_core();
        assert!(!core.is_loading());
        core.set_loading(true);
        assert!(core.is_loading());
        core.set_loading(false);
        assert!(!core.is_loading());
    }

    #[test]
    fn test_toggle_sort_cycles_orders() {
        let mut core = new_core();
        core.set_items(three_rows());
        core.toggle_sort();
        let names: Vec<&str> = core
            .filtered_items()
            .iter()
            .map(|row| row.name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
        core.toggle_sort();
        let names: Vec<&str> = core
            .filtered_items()
            .iter()
            .map(|row| row.name.as_str())
            .collect();
        assert_eq!(names, vec!["gamma", "beta", "alpha"]);
        core.toggle_sort();
        let names: Vec<&str> = core
            .filtered_items()
            .iter()
            .map(|row| row.name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_sort_survives_set_items_refresh() {
        let mut core = new_core();
        core.set_items(three_rows());
        core.toggle_sort();
        core.set_items(three_rows());
        let names: Vec<&str> = core
            .filtered_items()
            .iter()
            .map(|row| row.name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_active_filter_only_when_active() {
        let mut core = new_core();
        core.set_items(three_rows());
        assert_eq!(core.active_filter(), None);
        core.activate_filter();
        core.push_filter_char('a');
        assert_eq!(core.active_filter(), Some("a"));
    }
}
