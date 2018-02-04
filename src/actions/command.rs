use std::io;
use std::process::{Child, Command, Stdio};

use serde_yaml::Value;

use super::Action;

pub const ACTION_NAME: &'static str = "command";

/// An action that executes a shell command on context enter.
///
/// The launched process is killed when the context is left or
/// the `CommandAction` is dropped.
#[derive(Debug)]
pub struct CommandAction {
    child: Option<Child>,
    enter_command: Command,
    exit_command: Option<Command>,
}

impl CommandAction {
    pub fn new(enter_command: &str, exit_command: Option<&str>) -> Self {
        CommandAction {
            child: None,
            enter_command: Self::command_from_line(enter_command),
            exit_command: exit_command.map(Self::command_from_line),
        }
    }

    pub fn from_config(value: &Value) -> io::Result<Self> {
        match *value {
            Value::String(ref cmd) => Ok(Self::new(cmd.as_ref(), None)),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown configuration format"))
        }
    }

    fn command_from_line(line: &str) -> Command {
        let mut parts = line.trim()
            .split(" ")
            .filter(|part| part.len() > 0);
        let command_name = parts.next()
            .expect("Missing command name.");

        let mut command = Command::new(command_name);
        command.args(parts)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        command
    }
}

impl Action for CommandAction {
    fn enter(&mut self) -> io::Result<()> {
        self.child = Some(self.enter_command.spawn()?);

        Ok(())
    }

    fn leave(&mut self) -> io::Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill()?;
        }
        if let Some(ref mut cmd) = self.exit_command {
            cmd.spawn()?.wait()?;
        }

        Ok(())
    }
}

impl Drop for CommandAction {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn smoke() {
        execute_and_kill("true")
    }

    #[cfg(windows)]
    #[test]
    fn smoke() {
        execute_and_kill("cmd /C exit 0")
    }

    #[should_panic]
    #[test]
    fn smoke_fail() {
        execute_and_kill("this-is-a-nonexisting-process")
    }

    #[cfg(unix)]
    #[test]
    fn starting_space() {
        execute_and_kill(" true");
    }

    #[cfg(windows)]
    #[test]
    fn starting_space() {
        execute_and_kill(" cmd /C exit 0");
    }

    fn execute_and_kill(cmd: &str) {
        let mut action = CommandAction::new(cmd, None);

        action.enter().unwrap();
        action.leave().unwrap();
    }
}
