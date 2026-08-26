use std::io::{self, Stdout, stdout};

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

pub(super) trait TerminalControl {
    fn enable_raw_mode(&mut self) -> io::Result<()>;
    fn enter_alternate_screen(&mut self) -> io::Result<()>;
    fn hide_cursor(&mut self) -> io::Result<()>;
    fn show_cursor(&mut self) -> io::Result<()>;
    fn leave_alternate_screen(&mut self) -> io::Result<()>;
    fn disable_raw_mode(&mut self) -> io::Result<()>;
}

pub(super) struct CrosstermControl {
    output: Stdout,
}

impl CrosstermControl {
    pub(super) fn new() -> Self {
        Self { output: stdout() }
    }
}

impl TerminalControl for CrosstermControl {
    fn enable_raw_mode(&mut self) -> io::Result<()> {
        enable_raw_mode()
    }

    fn enter_alternate_screen(&mut self) -> io::Result<()> {
        execute!(self.output, EnterAlternateScreen)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        execute!(self.output, Hide)
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        execute!(self.output, Show)
    }

    fn leave_alternate_screen(&mut self) -> io::Result<()> {
        execute!(self.output, LeaveAlternateScreen)
    }

    fn disable_raw_mode(&mut self) -> io::Result<()> {
        disable_raw_mode()
    }
}

struct TerminalSession<C: TerminalControl> {
    control: C,
    raw_mode_enabled: bool,
    alternate_screen_entered: bool,
    cursor_hidden: bool,
}

impl<C: TerminalControl> TerminalSession<C> {
    fn start(control: C) -> io::Result<Self> {
        let mut session = Self {
            control,
            raw_mode_enabled: false,
            alternate_screen_entered: false,
            cursor_hidden: false,
        };

        session.control.enable_raw_mode()?;
        session.raw_mode_enabled = true;

        if let Err(error) = session.control.enter_alternate_screen() {
            let _ = session.restore();
            return Err(error);
        }
        session.alternate_screen_entered = true;

        if let Err(error) = session.control.hide_cursor() {
            let _ = session.restore();
            return Err(error);
        }
        session.cursor_hidden = true;

        Ok(session)
    }

    fn restore(&mut self) -> io::Result<()> {
        let mut first_error = None;

        if self.cursor_hidden {
            self.cursor_hidden = false;
            record_first_error(&mut first_error, self.control.show_cursor());
        }
        if self.alternate_screen_entered {
            self.alternate_screen_entered = false;
            record_first_error(&mut first_error, self.control.leave_alternate_screen());
        }
        if self.raw_mode_enabled {
            self.raw_mode_enabled = false;
            record_first_error(&mut first_error, self.control.disable_raw_mode());
        }

        first_error.map_or(Ok(()), Err)
    }
}

impl<C: TerminalControl> Drop for TerminalSession<C> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn record_first_error(first_error: &mut Option<io::Error>, result: io::Result<()>) {
    if let Err(error) = result
        && first_error.is_none()
    {
        *first_error = Some(error);
    }
}

pub(super) fn with_terminal<C, T>(
    control: C,
    operation: impl FnOnce() -> io::Result<T>,
) -> io::Result<T>
where
    C: TerminalControl,
{
    let mut session = TerminalSession::start(control)?;
    let operation_result = operation();
    let restore_result = session.restore();

    match operation_result {
        Ok(value) => restore_result.map(|()| value),
        Err(error) => {
            let _ = restore_result;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
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

        let error =
            with_terminal(control, || Err::<(), _>(io::Error::other("operation"))).unwrap_err();

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
}
