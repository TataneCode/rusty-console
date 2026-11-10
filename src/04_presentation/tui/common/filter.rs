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

    pub fn stop_input(&mut self) {
        self.active = false;
    }

    pub fn push_char(&mut self, c: char) {
        self.value.push(c);
    }

    pub fn push_str(&mut self, text: &str) {
        self.value.push_str(text);
    }

    pub fn pop_char(&mut self) {
        self.value.pop();
    }

    /// Clears the typed value without deactivating the filter.
    pub fn clear_value(&mut self) {
        self.value.clear();
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
    fn test_push_str_and_clear_value_keep_active_state() {
        let mut filter = FilterState::new();
        filter.activate();
        filter.push_char('a');
        filter.push_str("bc");
        assert_eq!(filter.value(), "abc");

        filter.clear_value();
        assert!(filter.value().is_empty());
        assert!(filter.is_active());
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
