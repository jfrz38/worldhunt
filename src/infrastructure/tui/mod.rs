mod app;
mod input;
mod layout;
mod map;
mod mvt;
mod terminal;
mod theme;

#[cfg(test)]
mod tests;

use std::{
    io::{self, stdout},
    sync::mpsc::{Receiver, TryRecvError},
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::domain::ports::{CountryCatalog, CountryProximity, TargetSelector};

use self::{
    app::TuiApp,
    terminal::{CrosstermControl, with_terminal},
};
use super::update_check::UpdateNotice;

const UPDATE_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub fn run<Catalog, ProximityPort, Selector>(
    catalog: &Catalog,
    proximity: &ProximityPort,
    selector: &mut Selector,
) -> io::Result<()>
where
    Catalog: CountryCatalog,
    ProximityPort: CountryProximity,
    Selector: TargetSelector,
{
    let updates = super::update_check::start(env!("CARGO_PKG_VERSION"));
    let app = TuiApp::new(catalog, proximity, selector)?;
    with_terminal(CrosstermControl::new(), || run_event_loop(app, updates))
}

/// Renders production wiring without requiring an interactive terminal.
pub fn render_initial_frame<Catalog, ProximityPort, Selector>(
    catalog: &Catalog,
    proximity: &ProximityPort,
    selector: &mut Selector,
    width: u16,
    height: u16,
) -> io::Result<()>
where
    Catalog: CountryCatalog,
    ProximityPort: CountryProximity,
    Selector: TargetSelector,
{
    let app = TuiApp::new(catalog, proximity, selector)?;
    let map = map::Map::load().map_err(io::Error::other)?;
    let backend = ratatui::backend::TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("TestBackend is infallible");
    terminal
        .draw(|frame| app.render(frame, &map))
        .expect("TestBackend draw is infallible");
    Ok(())
}

fn run_event_loop<Catalog, ProximityPort, Selector>(
    mut app: TuiApp<'_, Catalog, ProximityPort, Selector>,
    mut updates: Option<Receiver<UpdateNotice>>,
) -> io::Result<()>
where
    Catalog: CountryCatalog,
    ProximityPort: CountryProximity,
    Selector: TargetSelector,
{
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut map = map::Map::load().map_err(io::Error::other)?;
    draw(&mut terminal, &app, &map)?;

    loop {
        if settle_update(&mut app, &mut updates) {
            draw(&mut terminal, &app, &map)?;
        }
        if updates.is_some() && !event::poll(UPDATE_POLL_INTERVAL)? {
            continue;
        }
        let mut redraw = false;
        for action in pending_actions(event_action(event::read()?))? {
            if action != EventAction::Wait {
                redraw = true;
            }
            app.handle(action, &mut map);
        }
        if app.should_quit() {
            return Ok(());
        }
        if redraw {
            draw(&mut terminal, &app, &map)?;
        }
    }
}

fn settle_update<Catalog, ProximityPort, Selector>(
    app: &mut TuiApp<'_, Catalog, ProximityPort, Selector>,
    updates: &mut Option<Receiver<UpdateNotice>>,
) -> bool
where
    Catalog: CountryCatalog,
    ProximityPort: CountryProximity,
    Selector: TargetSelector,
{
    let Some(receiver) = updates.as_ref() else {
        return false;
    };
    match receiver.try_recv() {
        Ok(notice) => {
            app.set_update_version(notice.latest);
            *updates = None;
            true
        }
        Err(TryRecvError::Empty) => false,
        Err(TryRecvError::Disconnected) => {
            *updates = None;
            false
        }
    }
}

fn draw<Catalog, ProximityPort, Selector>(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &TuiApp<'_, Catalog, ProximityPort, Selector>,
    map: &map::Map,
) -> io::Result<()>
where
    Catalog: CountryCatalog,
    ProximityPort: CountryProximity,
    Selector: TargetSelector,
{
    terminal.draw(|frame| {
        app.render(frame, map);
    })?;
    Ok(())
}

#[derive(Debug, PartialEq)]
enum EventAction {
    Quit,
    Zoom(i8),
    ZoomOutOrInsertHyphen,
    Pan(f64, f64),
    Input(input::InputAction),
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
            KeyCode::Char('-') => EventAction::ZoomOutOrInsertHyphen,
            KeyCode::Char('_') => EventAction::Zoom(-1),
            KeyCode::Left => EventAction::Pan(-1.0, 0.0),
            KeyCode::Right => EventAction::Pan(1.0, 0.0),
            KeyCode::Up => EventAction::Pan(0.0, -1.0),
            KeyCode::Down => EventAction::Pan(0.0, 1.0),
            code => input::action_for(code).map_or(EventAction::Wait, EventAction::Input),
        },
        Event::Mouse(mouse) => match mouse.kind {
            MouseEventKind::ScrollUp => EventAction::Zoom(1),
            MouseEventKind::ScrollDown => EventAction::Zoom(-1),
            _ => EventAction::Wait,
        },
        _ => EventAction::Wait,
    }
}
