#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterState {
    value: String,
    active: bool,
}

impl FilterState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn active_value(&self) -> Option<&str> {
        self.active.then_some(self.value())
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.value.clear();
    }

    pub fn push_char(&mut self, c: char) {
        self.value.push(c);
    }

    pub fn pop_char(&mut self) {
        self.value.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::FilterState;

    #[test]
    fn test_active_value_only_when_filter_is_active() {
        let mut filter = FilterState::new();
        filter.push_char('a');
        assert_eq!(filter.active_value(), None);

        filter.activate();
        assert_eq!(filter.active_value(), Some("a"));
    }

    #[test]
    fn test_deactivate_clears_value() {
        let mut filter = FilterState::new();
        filter.activate();
        filter.push_char('a');
        filter.push_char('b');

        filter.deactivate();

        assert!(!filter.is_active());
        assert!(filter.value().is_empty());
    }
}
