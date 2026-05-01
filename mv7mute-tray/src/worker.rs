use crate::events::{UserEvent, WorkerCommand, WorkerEvent};
use mv7mute_core::{DeviceState, query_state, toggle_lock_state, toggle_mute_state};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;
use winit::event_loop::EventLoopProxy;

pub const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

pub fn spawn_worker(
    proxy: EventLoopProxy<UserEvent>,
) -> (Sender<WorkerCommand>, thread::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || worker_loop(proxy, rx));
    (tx, handle)
}

fn worker_loop(proxy: EventLoopProxy<UserEvent>, rx: Receiver<WorkerCommand>) {
    loop {
        match rx.recv_timeout(REFRESH_INTERVAL) {
            Ok(WorkerCommand::Refresh) => emit_query_result(&proxy, query_state()),
            Ok(WorkerCommand::Toggle) => emit_query_result(&proxy, toggle_mute_state()),
            Ok(WorkerCommand::ToggleLock) => emit_query_result(&proxy, toggle_lock_state()),
            Ok(WorkerCommand::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => emit_query_result(&proxy, query_state()),
        }
    }
}

fn emit_query_result(proxy: &EventLoopProxy<UserEvent>, result: Result<DeviceState, String>) {
    let event = match result {
        Ok(state) => WorkerEvent::StateUpdated(state),
        Err(error) => WorkerEvent::Error(error),
    };
    let _ = proxy.send_event(UserEvent::Worker(event));
}
