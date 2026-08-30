use crossterm::event::KeyCode;
use ratatui::{Terminal, backend::TestBackend};

use crate::{
    domain::{
        CountryId, GuessInput, Proximity,
        ports::{CountryCatalog, CountryProximity, TargetSelector},
    },
    infrastructure::tui::{EventAction, app::TuiApp, input, map::Map},
};

struct Catalog;

impl CountryCatalog for Catalog {
    fn playable(&self) -> &[CountryId] {
        static COUNTRIES: [CountryId; 2] = [CountryId::new(0), CountryId::new(1)];
        &COUNTRIES
    }

    fn name(&self, country: CountryId) -> Option<&str> {
        match country.value() {
            0 => Some("France"),
            1 => Some("Spain"),
            _ => None,
        }
    }

    fn resolve(&self, input: &GuessInput) -> Option<CountryId> {
        match input.as_str() {
            "France" => Some(CountryId::new(0)),
            "Spain" => Some(CountryId::new(1)),
            _ => None,
        }
    }
}

struct ProximityPort;

impl CountryProximity for ProximityPort {
    fn between(&self, _: CountryId, _: CountryId) -> Option<Proximity> {
        Some(Proximity::new(0, true))
    }
}

struct Selector(CountryId);

impl TargetSelector for Selector {
    fn select(&mut self, _: &[CountryId]) -> CountryId {
        self.0
    }
}

fn app<'a>(selector: &'a mut Selector) -> TuiApp<'a, Catalog, ProximityPort, Selector> {
    TuiApp::new(&Catalog, &ProximityPort, selector).expect("game starts")
}

#[test]
fn accepted_guess_clears_the_input_and_is_shown_in_history() {
    let mut selector = Selector(CountryId::new(0));
    let mut app = app(&mut selector);
    let mut map = Map::load().expect("embedded map is valid");

    for character in "Spain".chars() {
        app.handle(
            EventAction::Input(input::InputAction::Insert(character)),
            &mut map,
        );
    }
    app.handle(EventAction::Input(input::InputAction::Submit), &mut map);

    assert!(app.input.is_empty());
    assert_eq!(app.game.guesses().len(), 1);
    assert!(app.message.is_none());
}

#[test]
fn unknown_input_is_recoverable_and_preserves_editable_text() {
    let mut selector = Selector(CountryId::new(0));
    let mut app = app(&mut selector);
    let mut map = Map::load().expect("embedded map is valid");

    app.handle(
        EventAction::Input(input::InputAction::Insert('X')),
        &mut map,
    );
    app.handle(EventAction::Input(input::InputAction::Submit), &mut map);

    assert_eq!(app.input, "X");
    assert_eq!(
        app.message.as_deref(),
        Some("Unknown country. Try a canonical name or known alias.")
    );
    assert!(app.game.guesses().is_empty());
}

#[test]
fn winning_game_allows_n_to_start_a_fresh_game() {
    let mut selector = Selector(CountryId::new(0));
    let mut app = app(&mut selector);
    let mut map = Map::load().expect("embedded map is valid");

    for character in "France".chars() {
        app.handle(
            EventAction::Input(input::InputAction::Insert(character)),
            &mut map,
        );
    }
    app.handle(EventAction::Input(input::InputAction::Submit), &mut map);
    app.handle(
        EventAction::Input(input::InputAction::Insert('n')),
        &mut map,
    );

    assert!(app.game.guesses().is_empty());
    assert!(app.message.is_none());
}

#[test]
fn wide_frame_contains_history_and_input() {
    let mut selector = Selector(CountryId::new(0));
    let app = app(&mut selector);
    let map = Map::load().expect("embedded map is valid");
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).expect("test terminal starts");

    terminal
        .draw(|frame| app.render(frame, &map))
        .expect("frame renders");

    let buffer = terminal.backend().buffer();
    let content = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(content.contains("Attempts (0)"));
    assert!(content.contains("Guess a country"));
    assert!(content.contains("Enter: submit"));
    assert_eq!(
        input::action_for(KeyCode::Char('q')),
        Some(input::InputAction::Insert('q'))
    );
}

#[test]
fn narrow_frame_stacks_the_map_history_and_input() {
    let mut selector = Selector(CountryId::new(0));
    let app = app(&mut selector);
    let map = Map::load().expect("embedded map is valid");
    let mut terminal = Terminal::new(TestBackend::new(70, 24)).expect("test terminal starts");

    terminal
        .draw(|frame| app.render(frame, &map))
        .expect("frame renders");

    let content = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(content.contains("Attempts (0)"));
    assert!(content.contains("Guess a country"));
}

#[test]
fn too_small_frame_explains_how_to_recover() {
    let mut selector = Selector(CountryId::new(0));
    let app = app(&mut selector);
    let map = Map::load().expect("embedded map is valid");
    let mut terminal = Terminal::new(TestBackend::new(47, 19)).expect("test terminal starts");

    terminal
        .draw(|frame| app.render(frame, &map))
        .expect("frame renders");

    let content = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(content.contains("Resize terminal"));
}
