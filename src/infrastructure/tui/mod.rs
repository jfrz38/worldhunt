mod terminal;

#[cfg(test)]
mod tests;

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
