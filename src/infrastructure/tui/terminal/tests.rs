use std::{
    cell::RefCell,
    io,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};

use super::{TerminalControl, TerminalSession, with_terminal};

const ENABLE_RAW: &str = "enable_raw";
const ENTER_SCREEN: &str = "enter_screen";
const HIDE_CURSOR: &str = "hide_cursor";
const SHOW_CURSOR: &str = "show_cursor";
const LEAVE_SCREEN: &str = "leave_screen";
const DISABLE_RAW: &str = "disable_raw";

#[derive(Clone)]
struct FakeControl {
    state: Rc<RefCell<FakeState>>,
}

struct FakeState {
    calls: Vec<&'static str>,
    failures: Vec<&'static str>,
}

impl FakeControl {
    fn new(failures: &[&'static str]) -> Self {
        Self {
            state: Rc::new(RefCell::new(FakeState {
                calls: Vec::new(),
                failures: failures.to_vec(),
            })),
        }
    }

    fn calls(&self) -> Vec<&'static str> {
        self.state.borrow().calls.clone()
    }

    fn call(&self, name: &'static str) -> io::Result<()> {
        let mut state = self.state.borrow_mut();
        state.calls.push(name);
        if state.failures.contains(&name) {
            Err(io::Error::other(name))
        } else {
            Ok(())
        }
    }
}

impl TerminalControl for FakeControl {
    fn enable_raw_mode(&mut self) -> io::Result<()> {
        self.call(ENABLE_RAW)
    }

    fn enter_alternate_screen(&mut self) -> io::Result<()> {
        self.call(ENTER_SCREEN)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.call(HIDE_CURSOR)
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.call(SHOW_CURSOR)
    }

    fn leave_alternate_screen(&mut self) -> io::Result<()> {
        self.call(LEAVE_SCREEN)
    }

    fn disable_raw_mode(&mut self) -> io::Result<()> {
        self.call(DISABLE_RAW)
    }
}

#[test]
fn starts_and_restores_in_reverse_order() {
    let control = FakeControl::new(&[]);
    let observer = control.clone();

    with_terminal(control, || Ok(())).unwrap();

    assert_eq!(
        observer.calls(),
        [
            ENABLE_RAW,
            ENTER_SCREEN,
            HIDE_CURSOR,
            SHOW_CURSOR,
            LEAVE_SCREEN,
            DISABLE_RAW,
        ]
    );
}

#[test]
fn rolls_back_completed_steps_when_setup_fails() {
    let control = FakeControl::new(&[HIDE_CURSOR]);
    let observer = control.clone();

    let error = with_terminal(control, || Ok(())).unwrap_err();

    assert_eq!(error.to_string(), HIDE_CURSOR);
    assert_eq!(
        observer.calls(),
        [
            ENABLE_RAW,
            ENTER_SCREEN,
            HIDE_CURSOR,
            LEAVE_SCREEN,
            DISABLE_RAW,
        ]
    );
}

#[test]
fn restoration_is_idempotent() {
    let control = FakeControl::new(&[]);
    let observer = control.clone();
    let mut session = TerminalSession::start(control).unwrap();

    session.restore().unwrap();
    session.restore().unwrap();
    drop(session);

    assert_eq!(observer.calls().len(), 6);
}

#[test]
fn restoration_attempts_every_step_and_returns_the_first_error() {
    let control = FakeControl::new(&[SHOW_CURSOR, LEAVE_SCREEN, DISABLE_RAW]);
    let observer = control.clone();
    let mut session = TerminalSession::start(control).unwrap();

    let error = session.restore().unwrap_err();

    assert_eq!(error.to_string(), SHOW_CURSOR);
    assert_eq!(
        observer.calls(),
        [
            ENABLE_RAW,
            ENTER_SCREEN,
            HIDE_CURSOR,
            SHOW_CURSOR,
            LEAVE_SCREEN,
            DISABLE_RAW,
        ]
    );
}

#[test]
fn restores_after_an_operation_error() {
    let control = FakeControl::new(&[]);
    let observer = control.clone();

    let error = with_terminal(control, || Err::<(), _>(io::Error::other("operation"))).unwrap_err();

    assert_eq!(error.to_string(), "operation");
    assert_eq!(observer.calls().len(), 6);
}

#[test]
fn restores_while_unwinding_a_panic() {
    let control = FakeControl::new(&[]);
    let observer = control.clone();

    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = with_terminal(control, || -> io::Result<()> { panic!("operation panic") });
    }));

    assert!(result.is_err());
    assert_eq!(observer.calls().len(), 6);
}
