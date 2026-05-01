pub mod mv7;

use mv7::{Mv7, Transport};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceState {
    pub muted: bool,
    pub locked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

pub fn query_state() -> Result<DeviceState, String> {
    let mut mv7 = Mv7::open()?;
    query_state_with_mv7(&mut mv7)
}

pub fn toggle_mute_state() -> Result<DeviceState, String> {
    let mut mv7 = Mv7::open()?;
    toggle_mute_state_with_mv7(&mut mv7)
}

pub fn toggle_lock_state() -> Result<DeviceState, String> {
    let mut mv7 = Mv7::open()?;
    toggle_lock_state_with_mv7(&mut mv7)
}

fn with_device<T: Transport, R>(
    mv7: &mut Mv7<T>,
    action: impl FnOnce(&mut Mv7<T>) -> Result<R, String>,
) -> Result<R, String> {
    mv7.init()?;
    let result = action(mv7);
    merge_value_results(result, mv7.restore_lock())
}

fn run_with_mv7<T: Transport>(mv7: &mut Mv7<T>, command: Command) -> Result<Vec<String>, String> {
    with_device(mv7, |mv7| execute_command(mv7, command))
}

fn query_state_with_mv7<T: Transport>(mv7: &mut Mv7<T>) -> Result<DeviceState, String> {
    with_device(mv7, current_state)
}

fn toggle_mute_state_with_mv7<T: Transport>(mv7: &mut Mv7<T>) -> Result<DeviceState, String> {
    with_device(mv7, |mv7| {
        let muted = mv7.toggle()?;
        Ok(DeviceState {
            muted,
            locked: mv7.was_locked(),
        })
    })
}

fn toggle_lock_state_with_mv7<T: Transport>(mv7: &mut Mv7<T>) -> Result<DeviceState, String> {
    with_device(mv7, |mv7| {
        let desired_locked = !mv7.was_locked();
        // Query mute while the device is still unlocked (init always unlocks).
        // Locking first and then reading micMute is an untested code path on the MV7.
        let muted = mv7.get_mute()?;
        if desired_locked {
            mv7.set_lock(true)?;
        }
        mv7.cancel_restore_lock();
        Ok(DeviceState {
            muted,
            locked: desired_locked,
        })
    })
}

fn current_state<T: Transport>(mv7: &mut Mv7<T>) -> Result<DeviceState, String> {
    Ok(DeviceState {
        muted: mv7.get_mute()?,
        locked: mv7.was_locked(),
    })
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
            let state = current_state(mv7)?;
            Ok(vec![
                if state.muted { "muted" } else { "unmuted" }.to_string(),
                if state.locked { "locked" } else { "unlocked" }.to_string(),
            ])
        }
        Command::Lock => {
            mv7.set_lock(true)?;
            mv7.cancel_restore_lock();
            Ok(vec!["locked".to_string()])
        }
        Command::Unlock => {
            mv7.set_lock(false)?;
            mv7.cancel_restore_lock();
            Ok(vec!["unlocked".to_string()])
        }
        Command::Version => unreachable!("info commands are handled before device access"),
    }
}

fn merge_value_results<T>(
    value_result: Result<T, String>,
    restore_result: Result<(), String>,
) -> Result<T, String> {
    match (value_result, restore_result) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(restore_error)) => Err(format!("Failed to restore lock: {restore_error}")),
        (Err(value_error), Ok(())) => Err(value_error),
        (Err(value_error), Err(restore_error)) => Err(format!(
            "{value_error}; additionally failed to restore lock: {restore_error}"
        )),
    }
}

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
