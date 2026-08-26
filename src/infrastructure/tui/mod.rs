mod terminal;

use std::io::{self, stdout};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Alignment,
    widgets::{Block, Paragraph},
};

use self::terminal::{CrosstermControl, with_terminal};

pub fn run() -> io::Result<()> {
    with_terminal(CrosstermControl::new(), run_event_loop)
}

fn run_event_loop() -> io::Result<()> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    draw(&mut terminal)?;

    loop {
        match event_action(event::read()?) {
            EventAction::Quit => return Ok(()),
            EventAction::Redraw => draw(&mut terminal)?,
            EventAction::Wait => {}
        }
    }
}

fn draw(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> io::Result<()> {
    terminal.draw(|frame| {
        let area = frame.area();
        let content = format!(
            "WorldHunt\n\nTerminal dimensions: {} x {}\n\nPress Esc or Ctrl+C to exit",
            area.width, area.height
        );
        let placeholder = Paragraph::new(content)
            .alignment(Alignment::Center)
            .block(Block::bordered().title(" Project foundation "));
        frame.render_widget(placeholder, area);
    })?;
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
enum EventAction {
    Quit,
    Redraw,
    Wait,
}

fn event_action(event: Event) -> EventAction {
    match event {
        Event::Resize(_, _) => EventAction::Redraw,
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Esc => EventAction::Quit,
            KeyCode::Char(character)
                if character.eq_ignore_ascii_case(&'c')
                    && key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                EventAction::Quit
            }
            _ => EventAction::Wait,
        },
        _ => EventAction::Wait,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    use super::{EventAction, event_action};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[test]
    fn escape_and_control_c_quit() {
        assert_eq!(
            event_action(key(KeyCode::Esc, KeyModifiers::NONE)),
            EventAction::Quit
        );
        assert_eq!(
            event_action(key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            EventAction::Quit
        );
    }

    #[test]
    fn resize_requests_a_redraw() {
        assert_eq!(event_action(Event::Resize(100, 40)), EventAction::Redraw);
    }

    #[test]
    fn plain_q_does_not_quit() {
        assert_eq!(
            event_action(key(KeyCode::Char('q'), KeyModifiers::NONE)),
            EventAction::Wait
        );
    }

    #[test]
    fn key_release_does_not_quit() {
        let event = Event::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        });

        assert_eq!(event_action(event), EventAction::Wait);
    }
}
