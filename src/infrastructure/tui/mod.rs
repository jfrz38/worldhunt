mod map;
mod mvt;
mod terminal;

#[cfg(test)]
mod tests;

use std::io::{self, stdout};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{Terminal, backend::CrosstermBackend};

use self::terminal::{CrosstermControl, with_terminal};

pub fn run() -> io::Result<()> {
    with_terminal(CrosstermControl::new(), run_event_loop)
}

fn run_event_loop() -> io::Result<()> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut map = map::Map::load().map_err(io::Error::other)?;
    draw(&mut terminal, &map)?;

    loop {
        match event_action(event::read()?) {
            EventAction::Quit => return Ok(()),
            EventAction::ZoomIn => {
                map.zoom_in();
                draw(&mut terminal, &map)?;
            }
            EventAction::ZoomOut => {
                map.zoom_out();
                draw(&mut terminal, &map)?;
            }
            EventAction::Pan(horizontal, vertical) => {
                map.pan(horizontal, vertical);
                draw(&mut terminal, &map)?;
            }
            EventAction::Redraw => draw(&mut terminal, &map)?,
            EventAction::Wait => {}
        }
    }
}

fn draw(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    map: &map::Map,
) -> io::Result<()> {
    terminal.draw(|frame| {
        map.render(frame.area(), frame.buffer_mut());
    })?;
    Ok(())
}

#[derive(Debug, PartialEq)]
enum EventAction {
    Quit,
    ZoomIn,
    ZoomOut,
    Pan(f64, f64),
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
            KeyCode::Char('+') | KeyCode::Char('=') => EventAction::ZoomIn,
            KeyCode::Char('-') | KeyCode::Char('_') => EventAction::ZoomOut,
            KeyCode::Left | KeyCode::Char('h') => EventAction::Pan(-1.0, 0.0),
            KeyCode::Right | KeyCode::Char('l') => EventAction::Pan(1.0, 0.0),
            KeyCode::Up | KeyCode::Char('k') => EventAction::Pan(0.0, -1.0),
            KeyCode::Down | KeyCode::Char('j') => EventAction::Pan(0.0, 1.0),
            _ => EventAction::Wait,
        },
        _ => EventAction::Wait,
    }
}
