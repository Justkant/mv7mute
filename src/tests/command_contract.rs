use super::support::run_scripted;
use crate::{Command, run};

#[test]
fn toggle_restores_lock_when_device_started_locked() {
    let (result, state) = run_scripted(
        Command::Toggle,
        vec![
            Ok(Some("su=adm")),
            Ok(Some("dspBooted")),
            Ok(Some("lock=on")),
            Ok(Some("micMute=off")),
        ],
    );

    assert_eq!(result.unwrap(), vec!["muted"]);
    assert_eq!(
        *state.sent.borrow(),
        vec![
            "su adm",
            "bootDSP C",
            "lock",
            "lock off",
            "micMute",
            "micMute on",
            "lock on"
        ]
    );
}

#[test]
fn toggle_does_not_relock_when_device_started_unlocked() {
    let (result, state) = run_scripted(
        Command::Toggle,
        vec![
            Ok(Some("su=adm")),
            Ok(Some("dspBooted")),
            Ok(Some("lock=off")),
            Ok(Some("micMute=off")),
        ],
    );

    assert_eq!(result.unwrap(), vec!["muted"]);
    assert_eq!(
        *state.sent.borrow(),
        vec!["su adm", "bootDSP C", "lock", "micMute", "micMute on"]
    );
}

#[test]
fn unlock_leaves_device_unlocked() {
    let (result, state) = run_scripted(
        Command::Unlock,
        vec![
            Ok(Some("su=adm")),
            Ok(Some("dspBooted")),
            Ok(Some("lock=on")),
        ],
    );

    assert_eq!(result.unwrap(), vec!["unlocked"]);
    assert_eq!(
        *state.sent.borrow(),
        vec!["su adm", "bootDSP C", "lock", "lock off", "lock off"]
    );
}

#[test]
fn lock_leaves_device_locked_without_extra_restore() {
    let (result, state) = run_scripted(
        Command::Lock,
        vec![
            Ok(Some("su=adm")),
            Ok(Some("dspBooted")),
            Ok(Some("lock=on")),
        ],
    );

    assert_eq!(result.unwrap(), vec!["locked"]);
    assert_eq!(
        *state.sent.borrow(),
        vec!["su adm", "bootDSP C", "lock", "lock off", "lock on"]
    );
}

#[test]
fn status_restores_lock_and_reports_original_state() {
    let (result, state) = run_scripted(
        Command::Status,
        vec![
            Ok(Some("su=adm")),
            Ok(Some("dspBooted")),
            Ok(Some("lock=on")),
            Ok(Some("micMute=on")),
        ],
    );

    assert_eq!(result.unwrap(), vec!["muted", "locked"]);
    assert_eq!(
        state.sent.borrow().last().map(String::as_str),
        Some("lock on")
    );
}

#[test]
fn version_returns_package_version_without_device_io() {
    let result = run(Command::Version);

    assert_eq!(
        result.unwrap(),
        vec![format!("mv7mute {}", env!("CARGO_PKG_VERSION"))]
    );
}
