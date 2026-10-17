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
    Spawned,
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

pub const TERMINAL_EMULATOR_CANDIDATES: [&str; 8] = [
    "x-terminal-emulator",
    "gnome-terminal",
    "konsole",
    "xfce4-terminal",
    "kitty",
    "alacritty",
    "wezterm",
    "xterm",
];

#[cfg(windows)]
pub const TERMINAL_EMULATOR_CANDIDATES: [&str; 4] = [
    "wt.exe",
    "WindowsTerminal.exe",
    "cmd.exe",
    "powershell.exe",
];

pub fn find_terminal_emulator() -> Option<String> {
    if let Ok(terminal) = std::env::var("RUSTY_CONSOLE_TERMINAL") {
        if !terminal.is_empty() && is_binary_on_path_or_absolute(&terminal) {
            return Some(terminal);
        }
    }
    TERMINAL_EMULATOR_CANDIDATES
        .iter()
        .find(|candidate| which_binary(candidate).is_ok())
        .map(|candidate| candidate.to_string())
}

fn is_binary_on_path_or_absolute(program: &str) -> bool {
    let path = std::path::Path::new(program);
    if path.components().count() > 1 {
        return is_executable(path).unwrap_or(false);
    }
    which_binary(program).is_ok()
}

fn which_binary(program: &str) -> io::Result<std::path::PathBuf> {
    let exe_suffix = if cfg!(windows) { ".exe" } else { "" };
    let program = if program.ends_with(".exe") || exe_suffix.is_empty() {
        program.to_string()
    } else {
        format!("{program}{exe_suffix}")
    };
    let path_var =
        std::env::var_os("PATH").ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(&program);
        if is_executable(&candidate)? {
            return Ok(candidate);
        }
    }
    Err(io::Error::from(io::ErrorKind::NotFound))
}

fn is_executable(path: &std::path::Path) -> io::Result<bool> {
    Ok(path.is_file())
}


fn terminal_exec_args(program: &str) -> &'static [&'static str] {
    if cfg!(windows) {
        return &[];
    }
    if program.contains("gnome-terminal") {
        &["--"]
    } else {
        &["-e"]
    }
}

pub fn build_external_exec_command(
    config: &ExecCommandConfig,
    request: &PendingExec,
    terminal: &str,
) -> std::process::Command {
    let mut command = std::process::Command::new(terminal);
    if cfg!(windows) {
        command.arg("cmd");
        command.arg("/c");
        command.arg("start");
        let script = format!(
            "docker --host {} exec -it {} {}",
            shell_quote(config.docker_host()),
            request.container_id,
            request.shell.as_str()
        );
        command.arg(script);
    } else {
        command.args(terminal_exec_args(terminal));
        command
            .arg("docker")
            .args(["--host", config.docker_host()])
            .arg("exec")
            .arg("-it")
            .arg(request.container_id.as_str())
            .arg(request.shell.as_str());
    }
    command
}

fn shell_quote(value: &str) -> String {
    if value.contains(' ') {
        format!("\"{value}\"")
    } else {
        value.to_string()
    }
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

    #[test]
    fn test_build_external_exec_command_unix_args() {
        let config = ExecCommandConfig::new("unix:///var/run/docker.sock");
        let request = PendingExec {
            container_id: "abc123".to_string(),
            shell: ExecShell::Bash,
            refresh_target: ExecRefreshTarget::Containers,
        };
        if !cfg!(windows) {
            let command =
                build_external_exec_command(&config, &request, "x-terminal-emulator");
            let args = command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            assert_eq!(command.get_program(), std::ffi::OsStr::new("x-terminal-emulator"));
            assert_eq!(
                args,
                vec![
                    "-e",
                    "docker",
                    "--host",
                    "unix:///var/run/docker.sock",
                    "exec",
                    "-it",
                    "abc123",
                    "bash",
                ]
            );
        }
    }

    #[test]
    fn test_build_external_exec_command_gnome_terminal_separator() {
        let config = ExecCommandConfig::new("unix:///var/run/docker.sock");
        let request = PendingExec {
            container_id: "abc123".to_string(),
            shell: ExecShell::Sh,
            refresh_target: ExecRefreshTarget::Containers,
        };
        if !cfg!(windows) {
            let command = build_external_exec_command(&config, &request, "gnome-terminal");
            let args = command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            assert_eq!(args.first(), Some(&"--".to_string()));
            assert_eq!(args.get(1), Some(&"docker".to_string()));
        }
    }

    #[test]
    fn test_which_binary_finds_sh() {
        if !cfg!(windows) {
            assert!(which_binary("sh").is_ok());
        }
    }

    #[test]
    fn test_find_terminal_emulator_env_override_missing_binary() {
        std::env::set_var("RUSTY_CONSOLE_TERMINAL", "/nonexistent/terminal-xyz");
        let found = std::env::var("RUSTY_CONSOLE_TERMINAL").ok();
        assert!(found.is_some());
        let result = find_terminal_emulator();
        std::env::remove_var("RUSTY_CONSOLE_TERMINAL");
        let _ = result;
    }
}
