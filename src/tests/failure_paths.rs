use super::support::{FakeTransport, run_scripted};
use crate::mv7::Mv7;
use crate::{Command, run_with_mv7};

#[test]
fn toggle_read_failure_still_attempts_relock() {
    let (result, state) = run_scripted(
        Command::Toggle,
        vec![
            Ok(Some("su=adm")),
            Ok(Some("dspBooted")),
            Ok(Some("lock=on")),
            Err("read failed"),
        ],
    );

    assert_eq!(result.unwrap_err(), "read failed");
    assert_eq!(
        state.sent.borrow().last().map(String::as_str),
        Some("lock on")
    );
}

#[test]
fn toggle_write_failure_still_attempts_relock() {
    let (transport, state) = FakeTransport::scripted(vec![
        Ok(Some("su=adm")),
        Ok(Some("dspBooted")),
        Ok(Some("lock=on")),
        Ok(Some("micMute=off")),
    ]);
    state
        .send_failures
        .borrow_mut()
        .push(("micMute on".to_string(), "write failed".to_string()));

    let mut mv7 = Mv7::new(transport);
    let result = run_with_mv7(&mut mv7, Command::Toggle);

    assert_eq!(result.unwrap_err(), "write failed");
    assert_eq!(
        state.sent.borrow().last().map(String::as_str),
        Some("lock on")
    );
}

#[test]
fn relock_failure_is_reported_as_command_failure() {
    let (transport, state) = FakeTransport::scripted(vec![
        Ok(Some("su=adm")),
        Ok(Some("dspBooted")),
        Ok(Some("lock=on")),
        Ok(Some("micMute=off")),
    ]);
    state
        .send_failures
        .borrow_mut()
        .push(("lock on".to_string(), "relock failed".to_string()));

    let mut mv7 = Mv7::new(transport);
    let result = run_with_mv7(&mut mv7, Command::Toggle);

    assert_eq!(result.unwrap_err(), "Failed to restore lock: relock failed");
}

#[test]
fn command_and_relock_failures_are_both_reported() {
    let (transport, state) = FakeTransport::scripted(vec![
        Ok(Some("su=adm")),
        Ok(Some("dspBooted")),
        Ok(Some("lock=on")),
        Ok(Some("micMute=off")),
    ]);
    state.send_failures.borrow_mut().extend([
        ("micMute on".to_string(), "write failed".to_string()),
        ("lock on".to_string(), "relock failed".to_string()),
    ]);

    let mut mv7 = Mv7::new(transport);
    let result = run_with_mv7(&mut mv7, Command::Toggle);

    assert_eq!(
        result.unwrap_err(),
        "write failed; additionally failed to restore lock: relock failed"
    );
}
