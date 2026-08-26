use std::io::{self, Stdout, stdout};

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

#[cfg(test)]
mod tests;

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
