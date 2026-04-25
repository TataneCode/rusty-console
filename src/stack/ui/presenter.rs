use crate::stack::application::StackDto;
use crate::ui::common::TableSelection;

pub struct StackPresenter {
    pub stacks: Vec<StackDto>,
    pub selection: TableSelection,
    pub loading: bool,
    pub error: Option<String>,
    pub filter: String,
    pub filter_active: bool,
}

impl StackPresenter {
    pub fn new() -> Self {
        StackPresenter {
            stacks: Vec::new(),
            selection: TableSelection::new(),
            loading: false,
            error: None,
            filter: String::new(),
            filter_active: false,
        }
    }

    pub fn set_stacks(&mut self, stacks: Vec<StackDto>) {
        self.stacks = stacks;
        self.update_filtered_selection();
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn filtered_stacks(&self) -> Vec<&StackDto> {
        if self.filter.is_empty() {
            self.stacks.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.stacks
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }

    pub fn selected_stack(&self) -> Option<&StackDto> {
        let filtered = self.filtered_stacks();
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
        let count = self.filtered_stacks().len();
        self.selection.set_items(count);
    }
}

impl Default for StackPresenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stack(name: &str, total: usize) -> StackDto {
        StackDto {
            name: name.to_string(),
            container_count: total,
            running_count: 0,
            containers: vec![],
        }
    }

    fn three_stacks() -> Vec<StackDto> {
        vec![
            make_stack("app-frontend", 2),
            make_stack("app-backend", 3),
            make_stack("db-postgres", 1),
        ]
    }

    #[test]
    fn test_new_empty() {
        let p = StackPresenter::new();
        assert!(p.stacks.is_empty());
        assert!(p.selected_stack().is_none());
        assert!(!p.loading);
        assert!(p.error.is_none());
        assert!(p.filter.is_empty());
        assert!(!p.filter_active);
    }

    #[test]
    fn test_set_stacks_selects_first() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        assert_eq!(p.stacks.len(), 2);
        assert_eq!(p.selected_stack().unwrap().name, "app-a");
    }

    #[test]
    fn test_set_stacks_clears_error() {
        let mut p = StackPresenter::new();
        p.set_error("something failed".to_string());
        assert!(p.error.is_some());
        p.set_stacks(three_stacks());
        assert!(p.error.is_none());
    }

    #[test]
    fn test_navigate_down() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        p.navigate_down();
        assert_eq!(p.selected_stack().unwrap().name, "app-b");
    }

    #[test]
    fn test_navigate_wraps_around() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        p.navigate_down();
        p.navigate_down(); // wraps to 0
        assert_eq!(p.selected_stack().unwrap().name, "app-a");
    }

    #[test]
    fn test_navigate_up_from_start_wraps() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        p.navigate_up(); // wraps to last
        assert_eq!(p.selected_stack().unwrap().name, "app-b");
    }

    #[test]
    fn test_selected_stack_empty() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![]);
        assert!(p.selected_stack().is_none());
    }

    #[test]
    fn test_set_stacks_replaces_previous() {
        let mut p = StackPresenter::new();
        p.set_stacks(vec![make_stack("old", 1)]);
        p.set_stacks(vec![make_stack("new-a", 2), make_stack("new-b", 3)]);
        assert_eq!(p.stacks.len(), 2);
        assert_eq!(p.selected_stack().unwrap().name, "new-a");
    }

    #[test]
    fn test_filtered_stacks_no_filter() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        assert_eq!(p.filtered_stacks().len(), 3);
    }

    #[test]
    fn test_filtered_stacks_by_name() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        for c in "app".chars() {
            p.push_filter_char(c);
        }
        let filtered = p.filtered_stacks();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|s| s.name.contains("app")));
    }

    #[test]
    fn test_filtered_stacks_case_insensitive() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        for c in "DB".chars() {
            p.push_filter_char(c);
        }
        let filtered = p.filtered_stacks();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "db-postgres");
    }

    #[test]
    fn test_filtered_stacks_no_match() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        for c in "zzzzz".chars() {
            p.push_filter_char(c);
        }
        assert!(p.filtered_stacks().is_empty());
    }

    #[test]
    fn test_selected_stack_from_filtered_list() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        for c in "postgres".chars() {
            p.push_filter_char(c);
        }
        let selected = p.selected_stack();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "db-postgres");
    }

    #[test]
    fn test_activate_deactivate_filter() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        p.activate_filter();
        assert!(p.filter_active);
        p.push_filter_char('x');
        p.deactivate_filter();
        assert!(!p.filter_active);
        assert!(p.filter.is_empty());
        assert_eq!(p.filtered_stacks().len(), 3);
    }

    #[test]
    fn test_filter_narrows_selection() {
        let mut p = StackPresenter::new();
        p.set_stacks(three_stacks());
        p.navigate_down();
        p.navigate_down();
        assert_eq!(p.selection.selected(), Some(2));
        for c in "db".chars() {
            p.push_filter_char(c);
        }
        assert_eq!(p.filtered_stacks().len(), 1);
        assert_eq!(p.selection.selected(), Some(0));
    }
}
