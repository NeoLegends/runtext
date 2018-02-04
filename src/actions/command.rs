use std::io;
use std::process::{Child, Command, Stdio};

use futures::future;
use futures::prelude::*;
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
            Value::Mapping(ref mapping) => {
                let enter = mapping.get(&Value::String("enter".to_owned()))
                    .and_then(|v| v.as_str())
                    .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Missing enter command key."))?;
                let exit = mapping.get(&Value::String("leave".to_owned()))
                    .and_then(|v| v.as_str());

                Ok(Self::new(enter, exit))
            },
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

    fn enter_impl(&mut self) -> io::Result<()> {
        self.child = Some(self.enter_command.spawn()?);

        Ok(())
    }

    fn leave_impl(&mut self) -> io::Result<()> {
        if let Some(mut child) = self.child.take() {
            child.kill()?;
        }
        if let Some(ref mut cmd) = self.exit_command {
            cmd.spawn()?.wait()?;
        }

        Ok(())
    }
}

impl Action for CommandAction {
    fn enter(&mut self) -> Box<Future<Item = (), Error = io::Error>> {
        Box::new(future::result(self.enter_impl()))
    }

    fn leave(&mut self) -> Box<Future<Item = (), Error = io::Error>> {
        Box::new(future::result(self.leave_impl()))
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
    use serde_yaml::Mapping;

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

        action.enter().wait().unwrap();
        action.leave().wait().unwrap();
    }

    #[test]
    fn load_cfg() {
        let cfg = Value::String("enter".to_owned());
        CommandAction::from_config(&cfg).unwrap();

        let enter_k = Value::String("enter".to_owned());
        let leave_k = Value::String("leave".to_owned());
        let mut map = Mapping::new();
        map.insert(enter_k, Value::String("enter".to_owned()));
        map.insert(leave_k, Value::String("enter".to_owned()));
        let cfg2 = Value::Mapping(map);
        CommandAction::from_config(&cfg2).unwrap();
    }

    #[test]
    fn load_cfg_map_empty_leave() {
        let cfg = Value::String("enter".to_owned());
        CommandAction::from_config(&cfg).unwrap();

        let enter_k = Value::String("enter".to_owned());
        let mut map = Mapping::new();
        map.insert(enter_k, Value::String("enter".to_owned()));
        let cfg2 = Value::Mapping(map);
        CommandAction::from_config(&cfg2).unwrap();
    }

    #[test]
    #[should_panic]
    fn load_cfg_fail1() {
        let cfg = Value::String("".to_owned());
        CommandAction::from_config(&cfg).unwrap();
    }

    #[test]
    #[should_panic]
    fn load_cfg_fail2() {
        let enter_k = Value::String("enter".to_owned());
        let leave_k = Value::String("leave".to_owned());
        let mut map = Mapping::new();
        map.insert(enter_k, Value::String("".to_owned()));
        map.insert(leave_k, Value::String("enter".to_owned()));
        let cfg2 = Value::Mapping(map);
        CommandAction::from_config(&cfg2).unwrap();
    }
}
