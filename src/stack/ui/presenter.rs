use crate::stack::application::StackDto;
use crate::ui::common::TableSelection;

pub struct StackPresenter {
    pub stacks: Vec<StackDto>,
    pub selection: TableSelection,
}

impl StackPresenter {
    pub fn new() -> Self {
        StackPresenter {
            stacks: Vec::new(),
            selection: TableSelection::new(),
        }
    }

    pub fn load(&mut self, stacks: Vec<StackDto>) {
        self.stacks = stacks;
        self.selection.set_items(self.stacks.len());
    }

    pub fn selected_stack(&self) -> Option<&StackDto> {
        self.selection.selected().and_then(|i| self.stacks.get(i))
    }

    pub fn navigate_up(&mut self) {
        self.selection.previous();
    }

    pub fn navigate_down(&mut self) {
        self.selection.next();
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

    #[test]
    fn test_new_empty() {
        let p = StackPresenter::new();
        assert!(p.stacks.is_empty());
        assert!(p.selected_stack().is_none());
    }

    #[test]
    fn test_load_selects_first() {
        let mut p = StackPresenter::new();
        p.load(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        assert_eq!(p.stacks.len(), 2);
        assert_eq!(p.selected_stack().unwrap().name, "app-a");
    }

    #[test]
    fn test_navigate_down() {
        let mut p = StackPresenter::new();
        p.load(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        p.navigate_down();
        assert_eq!(p.selected_stack().unwrap().name, "app-b");
    }

    #[test]
    fn test_navigate_wraps_around() {
        let mut p = StackPresenter::new();
        p.load(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        p.navigate_down();
        p.navigate_down(); // wraps to 0
        assert_eq!(p.selected_stack().unwrap().name, "app-a");
    }

    #[test]
    fn test_navigate_up_from_start_wraps() {
        let mut p = StackPresenter::new();
        p.load(vec![make_stack("app-a", 2), make_stack("app-b", 1)]);
        p.navigate_up(); // wraps to last
        assert_eq!(p.selected_stack().unwrap().name, "app-b");
    }

    #[test]
    fn test_selected_stack_empty() {
        let mut p = StackPresenter::new();
        p.load(vec![]);
        assert!(p.selected_stack().is_none());
    }

    #[test]
    fn test_load_replaces_previous() {
        let mut p = StackPresenter::new();
        p.load(vec![make_stack("old", 1)]);
        p.load(vec![make_stack("new-a", 2), make_stack("new-b", 3)]);
        assert_eq!(p.stacks.len(), 2);
        assert_eq!(p.selected_stack().unwrap().name, "new-a");
    }
}
