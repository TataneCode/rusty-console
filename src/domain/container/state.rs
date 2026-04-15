use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerState {
    Running,
    Paused,
    Stopped,
    Exited,
    Dead,
    Created,
    Removing,
    Restarting,
}

impl FromStr for ContainerState {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "running" => ContainerState::Running,
            "paused" => ContainerState::Paused,
            "exited" => ContainerState::Exited,
            "dead" => ContainerState::Dead,
            "created" => ContainerState::Created,
            "removing" => ContainerState::Removing,
            "restarting" => ContainerState::Restarting,
            _ => ContainerState::Stopped,
        })
    }
}

impl ContainerState {
    pub fn is_running(&self) -> bool {
        matches!(self, ContainerState::Running)
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self,
            ContainerState::Running | ContainerState::Paused | ContainerState::Restarting
        )
    }

    pub fn can_be_started(&self) -> bool {
        matches!(
            self,
            ContainerState::Stopped | ContainerState::Exited | ContainerState::Created
        )
    }

    pub fn can_be_stopped(&self) -> bool {
        matches!(
            self,
            ContainerState::Running | ContainerState::Paused | ContainerState::Restarting
        )
    }

    pub fn can_be_paused(&self) -> bool {
        matches!(self, ContainerState::Running)
    }

    pub fn can_be_unpaused(&self) -> bool {
        matches!(self, ContainerState::Paused)
    }

    pub fn can_be_restarted(&self) -> bool {
        matches!(
            self,
            ContainerState::Running | ContainerState::Paused | ContainerState::Restarting
        )
    }

    pub fn can_be_deleted(&self) -> bool {
        !self.is_active()
    }
}

impl fmt::Display for ContainerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerState::Running => write!(f, "Running"),
            ContainerState::Paused => write!(f, "Paused"),
            ContainerState::Stopped => write!(f, "Stopped"),
            ContainerState::Exited => write!(f, "Exited"),
            ContainerState::Dead => write!(f, "Dead"),
            ContainerState::Created => write!(f, "Created"),
            ContainerState::Removing => write!(f, "Removing"),
            ContainerState::Restarting => write!(f, "Restarting"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            "running".parse::<ContainerState>().unwrap(),
            ContainerState::Running
        );
        assert_eq!(
            "RUNNING".parse::<ContainerState>().unwrap(),
            ContainerState::Running
        );
        assert_eq!(
            "exited".parse::<ContainerState>().unwrap(),
            ContainerState::Exited
        );
        assert_eq!(
            "unknown".parse::<ContainerState>().unwrap(),
            ContainerState::Stopped
        );
    }

    #[test]
    fn test_can_be_started() {
        assert!(ContainerState::Stopped.can_be_started());
        assert!(ContainerState::Exited.can_be_started());
        assert!(ContainerState::Created.can_be_started());
        assert!(!ContainerState::Running.can_be_started());
    }

    #[test]
    fn test_can_be_stopped() {
        assert!(ContainerState::Running.can_be_stopped());
        assert!(ContainerState::Paused.can_be_stopped());
        assert!(!ContainerState::Stopped.can_be_stopped());
        assert!(!ContainerState::Exited.can_be_stopped());
    }

    #[test]
    fn test_can_be_restarted() {
        assert!(ContainerState::Running.can_be_restarted());
        assert!(ContainerState::Paused.can_be_restarted());
        assert!(ContainerState::Restarting.can_be_restarted());
        assert!(!ContainerState::Stopped.can_be_restarted());
        assert!(!ContainerState::Exited.can_be_restarted());
        assert!(!ContainerState::Created.can_be_restarted());
        assert!(!ContainerState::Dead.can_be_restarted());
        assert!(!ContainerState::Removing.can_be_restarted());
    }

    #[test]
    fn test_can_be_paused() {
        assert!(ContainerState::Running.can_be_paused());
        assert!(!ContainerState::Paused.can_be_paused());
        assert!(!ContainerState::Stopped.can_be_paused());
        assert!(!ContainerState::Exited.can_be_paused());
        assert!(!ContainerState::Dead.can_be_paused());
        assert!(!ContainerState::Created.can_be_paused());
        assert!(!ContainerState::Removing.can_be_paused());
        assert!(!ContainerState::Restarting.can_be_paused());
    }

    #[test]
    fn test_can_be_unpaused() {
        assert!(ContainerState::Paused.can_be_unpaused());
        assert!(!ContainerState::Running.can_be_unpaused());
        assert!(!ContainerState::Stopped.can_be_unpaused());
        assert!(!ContainerState::Exited.can_be_unpaused());
        assert!(!ContainerState::Dead.can_be_unpaused());
        assert!(!ContainerState::Created.can_be_unpaused());
        assert!(!ContainerState::Removing.can_be_unpaused());
        assert!(!ContainerState::Restarting.can_be_unpaused());
    }
}
