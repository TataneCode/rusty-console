use ratatui::widgets::ListState;
use std::io;

pub const EXEC_SHELL_OPTIONS: [&str; 2] = ["sh", "bash"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecCommandConfig {
    docker_host: String,
}

impl ExecCommandConfig {
    pub fn new(docker_host: impl Into<String>) -> Self {
        ExecCommandConfig {
            docker_host: docker_host.into(),
        }
    }

    pub fn docker_host(&self) -> &str {
        &self.docker_host
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecShell {
    Sh,
    Bash,
}

impl ExecShell {
    pub fn as_str(self) -> &'static str {
        match self {
            ExecShell::Sh => "sh",
            ExecShell::Bash => "bash",
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            1 => ExecShell::Bash,
            _ => ExecShell::Sh,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecRefreshTarget {
    Containers,
    StackContainers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingExec {
    pub container_id: String,
    pub shell: ExecShell,
    pub refresh_target: ExecRefreshTarget,
}

#[derive(Debug)]
pub enum ExecCommandResult {
    Status(std::process::ExitStatus),
    SpawnError(io::Error),
}

#[derive(Debug)]
pub struct ExecShellDialog {
    container_id: String,
    refresh_target: ExecRefreshTarget,
    state: ListState,
}

impl ExecShellDialog {
    pub fn new(container_id: String, refresh_target: ExecRefreshTarget) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        ExecShellDialog {
            container_id,
            refresh_target,
            state,
        }
    }

    pub fn state_mut(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn navigate_up(&mut self) {
        self.select_previous();
    }

    pub fn navigate_down(&mut self) {
        self.select_next();
    }

    pub fn build_request(&self) -> PendingExec {
        PendingExec {
            container_id: self.container_id.clone(),
            shell: ExecShell::from_index(self.state.selected().unwrap_or(0)),
            refresh_target: self.refresh_target,
        }
    }

    fn select_next(&mut self) {
        let selected = self.state.selected().unwrap_or(0);
        let next = if selected + 1 >= EXEC_SHELL_OPTIONS.len() {
            0
        } else {
            selected + 1
        };
        self.state.select(Some(next));
    }

    fn select_previous(&mut self) {
        let selected = self.state.selected().unwrap_or(0);
        let previous = if selected == 0 {
            EXEC_SHELL_OPTIONS.len() - 1
        } else {
            selected - 1
        };
        self.state.select(Some(previous));
    }
}

pub fn build_exec_command(
    config: &ExecCommandConfig,
    request: &PendingExec,
) -> std::process::Command {
    let mut command = std::process::Command::new("docker");
    command
        .args(["--host", config.docker_host()])
        .arg("exec")
        .arg("-it")
        .arg(request.container_id.as_str())
        .arg(request.shell.as_str());
    command
}

pub fn format_exec_error(error: &io::Error) -> String {
    match error.kind() {
        io::ErrorKind::NotFound => format!(
            "Docker CLI not found on PATH. Install the `docker` binary to use Exec. Raw error: {error}"
        ),
        io::ErrorKind::PermissionDenied => format!(
            "Docker CLI exists but is not executable. Check the `docker` binary permissions to use Exec. Raw error: {error}"
        ),
        _ => format!("Failed to start `docker exec`. Raw error: {error}"),
    }
}

pub fn finalize_exec_command_result(
    exec_result: io::Result<std::process::ExitStatus>,
    resume_result: io::Result<()>,
) -> io::Result<ExecCommandResult> {
    match (exec_result, resume_result) {
        (Ok(status), Ok(())) => Ok(ExecCommandResult::Status(status)),
        (Err(exec_err), Ok(())) => Ok(ExecCommandResult::SpawnError(exec_err)),
        (Ok(_), Err(resume_err)) => Err(resume_err),
        (Err(exec_err), Err(resume_err)) => Err(io::Error::new(
            resume_err.kind(),
            format!("{resume_err}; docker exec also failed: {exec_err}"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_shell_as_str() {
        assert_eq!(ExecShell::Sh.as_str(), "sh");
        assert_eq!(ExecShell::Bash.as_str(), "bash");
    }

    #[test]
    fn test_exec_shell_from_index() {
        assert_eq!(ExecShell::from_index(0), ExecShell::Sh);
        assert_eq!(ExecShell::from_index(1), ExecShell::Bash);
        assert_eq!(ExecShell::from_index(7), ExecShell::Sh);
    }

    #[test]
    fn test_exec_command_config_docker_host() {
        let config = ExecCommandConfig::new("unix:///var/run/docker.sock");
        assert_eq!(config.docker_host(), "unix:///var/run/docker.sock");
    }

    #[test]
    fn test_exec_shell_dialog_navigation() {
        let mut dialog = ExecShellDialog::new("abc123".to_string(), ExecRefreshTarget::Containers);
        dialog.navigate_down();
        dialog.navigate_down();
        let request = dialog.build_request();
        assert_eq!(request.shell, ExecShell::Sh);
        assert_eq!(request.container_id, "abc123");
        assert_eq!(request.refresh_target, ExecRefreshTarget::Containers);
    }

    #[test]
    fn test_build_exec_command_args() {
        let config = ExecCommandConfig::new("unix:///var/run/docker.sock");
        let request = PendingExec {
            container_id: "abc123".to_string(),
            shell: ExecShell::Bash,
            refresh_target: ExecRefreshTarget::Containers,
        };
        let command = build_exec_command(&config, &request);
        assert_eq!(
            format!("{command:?}"),
            "\"docker\" \"--host\" \"unix:///var/run/docker.sock\" \"exec\" \"-it\" \"abc123\" \"bash\""
        );
    }

    #[test]
    fn test_format_exec_error_not_found() {
        let error = io::Error::from(io::ErrorKind::NotFound);
        assert!(format_exec_error(&error).contains("not found on PATH"));
    }

    #[test]
    fn test_finalize_exec_command_result_spawn_error() {
        let error = io::Error::from(io::ErrorKind::NotFound);
        let result = finalize_exec_command_result(Err(error), Ok(()));
        assert!(matches!(result, Ok(ExecCommandResult::SpawnError(_))));
    }
}
