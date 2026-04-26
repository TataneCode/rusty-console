use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackContainerState {
    Running,
    Paused,
    Stopped,
    Exited,
    Dead,
    Created,
    Removing,
    Restarting,
}

impl FromStr for StackContainerState {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "running" => StackContainerState::Running,
            "paused" => StackContainerState::Paused,
            "exited" => StackContainerState::Exited,
            "dead" => StackContainerState::Dead,
            "created" => StackContainerState::Created,
            "removing" => StackContainerState::Removing,
            "restarting" => StackContainerState::Restarting,
            _ => StackContainerState::Stopped,
        })
    }
}

impl StackContainerState {
    pub fn is_running(&self) -> bool {
        matches!(self, StackContainerState::Running)
    }

    pub fn can_be_started(&self) -> bool {
        matches!(
            self,
            StackContainerState::Stopped
                | StackContainerState::Exited
                | StackContainerState::Created
        )
    }

    pub fn can_be_stopped(&self) -> bool {
        matches!(
            self,
            StackContainerState::Running
                | StackContainerState::Paused
                | StackContainerState::Restarting
        )
    }
}

impl fmt::Display for StackContainerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackContainerState::Running => write!(f, "Running"),
            StackContainerState::Paused => write!(f, "Paused"),
            StackContainerState::Stopped => write!(f, "Stopped"),
            StackContainerState::Exited => write!(f, "Exited"),
            StackContainerState::Dead => write!(f, "Dead"),
            StackContainerState::Created => write!(f, "Created"),
            StackContainerState::Removing => write!(f, "Removing"),
            StackContainerState::Restarting => write!(f, "Restarting"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_defaults_unknown_to_stopped() {
        assert_eq!(
            "unknown".parse::<StackContainerState>().unwrap(),
            StackContainerState::Stopped
        );
    }

    #[test]
    fn test_running_can_be_stopped() {
        assert!(StackContainerState::Running.can_be_stopped());
        assert!(!StackContainerState::Running.can_be_started());
    }

    #[test]
    fn test_stopped_can_be_started() {
        assert!(StackContainerState::Stopped.can_be_started());
        assert!(!StackContainerState::Stopped.can_be_stopped());
    }
}
