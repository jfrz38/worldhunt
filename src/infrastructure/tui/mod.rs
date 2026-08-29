mod layout;
mod map;
mod mvt;
mod terminal;
mod theme;

#[cfg(test)]
mod tests;

use std::{
    io::{self, stdout},
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
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
        let mut redraw = false;
        for action in pending_actions(event_action(event::read()?))? {
            match action {
                EventAction::Quit => return Ok(()),
                EventAction::Zoom(steps) => {
                    for _ in 0..steps.unsigned_abs() {
                        if steps.is_positive() {
                            map.zoom_in();
                        } else {
                            map.zoom_out();
                        }
                    }
                    redraw = true;
                }
                EventAction::Pan(horizontal, vertical) => {
                    map.pan(horizontal, vertical);
                    redraw = true;
                }
                EventAction::Redraw => redraw = true,
                EventAction::Wait => {}
            }
        }
        if redraw {
            draw(&mut terminal, &map)?;
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
    Zoom(i8),
    Pan(f64, f64),
    Redraw,
    Wait,
}

fn pending_actions(action: EventAction) -> io::Result<Vec<EventAction>> {
    let mut actions = vec![action];
    while event::poll(Duration::ZERO)? {
        actions.push(event_action(event::read()?));
    }
    Ok(actions)
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
            KeyCode::Char('+') | KeyCode::Char('=') => EventAction::Zoom(1),
            KeyCode::Char('-') | KeyCode::Char('_') => EventAction::Zoom(-1),
            KeyCode::Left | KeyCode::Char('h') => EventAction::Pan(-1.0, 0.0),
            KeyCode::Right | KeyCode::Char('l') => EventAction::Pan(1.0, 0.0),
            KeyCode::Up | KeyCode::Char('k') => EventAction::Pan(0.0, -1.0),
            KeyCode::Down | KeyCode::Char('j') => EventAction::Pan(0.0, 1.0),
            _ => EventAction::Wait,
        },
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::ScrollUp => EventAction::Zoom(1),
            MouseEventKind::ScrollDown => EventAction::Zoom(-1),
            _ => EventAction::Wait,
        },
        _ => EventAction::Wait,
    }
}
