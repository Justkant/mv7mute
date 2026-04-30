use crate::mv7::{Mv7, Transport};
use crate::{run_with_mv7, Command};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Default)]
pub(super) struct FakeState {
    pub(super) sent: RefCell<Vec<String>>,
    pub(super) reads: RefCell<VecDeque<Result<Option<String>, String>>>,
    pub(super) send_failures: RefCell<Vec<(String, String)>>,
}

pub(super) struct FakeTransport {
    state: Rc<FakeState>,
}

impl FakeTransport {
    pub(super) fn scripted(reads: Vec<Result<Option<&str>, &str>>) -> (Self, Rc<FakeState>) {
        let state = Rc::new(FakeState {
            sent: RefCell::new(Vec::new()),
            reads: RefCell::new(
                reads
                    .into_iter()
                    .map(|entry| match entry {
                        Ok(Some(value)) => Ok(Some(value.to_string())),
                        Ok(None) => Ok(None),
                        Err(error) => Err(error.to_string()),
                    })
                    .collect(),
            ),
            send_failures: RefCell::new(Vec::new()),
        });
        (
            Self {
                state: Rc::clone(&state),
            },
            state,
        )
    }
}

impl Transport for FakeTransport {
    fn send(&self, cmd: &str) -> Result<(), String> {
        self.state.sent.borrow_mut().push(cmd.to_string());

        let failure_index = {
            let send_failures = self.state.send_failures.borrow();
            send_failures
                .iter()
                .position(|(failed_cmd, _)| failed_cmd == cmd)
        };

        if let Some(index) = failure_index {
            let (_, error) = self.state.send_failures.borrow_mut().remove(index);
            return Err(error);
        }

        Ok(())
    }

    fn read(&self, _timeout_ms: i32) -> Result<Option<String>, String> {
        self.state
            .reads
            .borrow_mut()
            .pop_front()
            .unwrap_or(Ok(None))
    }
}

pub(super) fn run_scripted(
    command: Command,
    reads: Vec<Result<Option<&'static str>, &'static str>>,
) -> (Result<Vec<String>, String>, Rc<FakeState>) {
    let (transport, state) = FakeTransport::scripted(reads);
    let mut mv7 = Mv7::new(transport);
    let result = run_with_mv7(&mut mv7, command);
    (result, state)
}