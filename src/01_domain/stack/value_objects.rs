use crate::domain::error::DomainError;

pub const STANDALONE: &str = "(standalone)";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StackName(String);

impl StackName {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainError> {
        let name = name.into();
        if name.is_empty() {
            return Err(DomainError::InvalidStackName(
                "Stack name cannot be empty".to_string(),
            ));
        }
        Ok(StackName(name))
    }

    pub fn standalone() -> Self {
        StackName(STANDALONE.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_standalone(&self) -> bool {
        self.0 == STANDALONE
    }
}

impl std::fmt::Display for StackName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid() {
        let name = StackName::new("my-app").unwrap();
        assert_eq!(name.as_str(), "my-app");
    }

    #[test]
    fn test_new_empty_fails() {
        assert!(StackName::new("").is_err());
    }

    #[test]
    fn test_standalone() {
        let name = StackName::standalone();
        assert!(name.is_standalone());
        assert_eq!(name.as_str(), STANDALONE);
    }

    #[test]
    fn test_non_standalone() {
        let name = StackName::new("my-app").unwrap();
        assert!(!name.is_standalone());
    }
}
