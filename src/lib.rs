pub mod mv7;

use clap::Subcommand;
use mv7::{Mv7, Transport};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Toggle mute state (default)
    Toggle,
    /// Mute the microphone
    On,
    /// Unmute the microphone
    Off,
    /// Print current mute state
    Status,
    /// Lock the device
    Lock,
    /// Unlock the device
    Unlock,
    /// Print version information
    Version,
}

impl Command {
    fn output_without_device(self) -> Option<Vec<String>> {
        match self {
            Self::Version => Some(vec![format!("mv7mute {}", env!("CARGO_PKG_VERSION"))]),
            _ => None,
        }
    }
}

pub fn run(command: Command) -> Result<Vec<String>, String> {
    if let Some(lines) = command.output_without_device() {
        return Ok(lines);
    }

    let mut mv7 = Mv7::open()?;
    run_with_mv7(&mut mv7, command)
}

fn run_with_mv7<T: Transport>(mv7: &mut Mv7<T>, command: Command) -> Result<Vec<String>, String> {
    mv7.init()?;

    let command_result = execute_command(mv7, command);
    let restore_result =
        if command_result.is_ok() && matches!(command, Command::Lock | Command::Unlock) {
            mv7.cancel_restore_lock();
            Ok(())
        } else {
            mv7.restore_lock()
        };

    merge_results(command_result, restore_result)
}

fn execute_command<T: Transport>(
    mv7: &mut Mv7<T>,
    command: Command,
) -> Result<Vec<String>, String> {
    match command {
        Command::Toggle => {
            let muted = mv7.toggle()?;
            Ok(vec![if muted { "muted" } else { "unmuted" }.to_string()])
        }
        Command::On => {
            mv7.set_mute(true)?;
            Ok(vec!["muted".to_string()])
        }
        Command::Off => {
            mv7.set_mute(false)?;
            Ok(vec!["unmuted".to_string()])
        }
        Command::Status => {
            let muted = mv7.get_mute()?;
            let locked = mv7.was_locked();
            Ok(vec![
                if muted { "muted" } else { "unmuted" }.to_string(),
                if locked { "locked" } else { "unlocked" }.to_string(),
            ])
        }
        Command::Lock => {
            mv7.set_lock(true)?;
            Ok(vec!["locked".to_string()])
        }
        Command::Unlock => {
            mv7.set_lock(false)?;
            Ok(vec!["unlocked".to_string()])
        }
        Command::Version => unreachable!("info commands are handled before device access"),
    }
}

fn merge_results(
    command_result: Result<Vec<String>, String>,
    restore_result: Result<(), String>,
) -> Result<Vec<String>, String> {
    match (command_result, restore_result) {
        (Ok(lines), Ok(())) => Ok(lines),
        (Ok(_), Err(restore_error)) => Err(format!("Failed to restore lock: {restore_error}")),
        (Err(command_error), Ok(())) => Err(command_error),
        (Err(command_error), Err(restore_error)) => Err(format!(
            "{command_error}; additionally failed to restore lock: {restore_error}"
        )),
    }
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
