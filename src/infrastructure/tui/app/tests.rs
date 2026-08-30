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

    fn search(&self, input: &GuessInput, _: usize) -> Vec<CountryId> {
        (input.as_str() == "Sp")
            .then_some(CountryId::new(1))
            .into_iter()
            .collect()
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
fn tab_completes_the_first_suggestion() {
    let mut selector = Selector(CountryId::new(0));
    let mut app = app(&mut selector);
    let mut map = Map::load().expect("embedded map is valid");

    for character in "Sp".chars() {
        app.handle(
            EventAction::Input(input::InputAction::Insert(character)),
            &mut map,
        );
    }
    app.handle(EventAction::Input(input::InputAction::Complete), &mut map);

    assert_eq!(app.input, "Spain");
}

#[test]
fn tab_completion_clears_a_previous_recoverable_message() {
    let mut selector = Selector(CountryId::new(0));
    let mut app = app(&mut selector);
    let mut map = Map::load().expect("embedded map is valid");

    app.handle(
        EventAction::Input(input::InputAction::Insert('X')),
        &mut map,
    );
    app.handle(EventAction::Input(input::InputAction::Submit), &mut map);
    app.handle(EventAction::Input(input::InputAction::Backspace), &mut map);
    for character in "Sp".chars() {
        app.handle(
            EventAction::Input(input::InputAction::Insert(character)),
            &mut map,
        );
    }
    app.handle(EventAction::Input(input::InputAction::Complete), &mut map);

    assert_eq!(app.input, "Spain");
    assert!(app.message.is_none());
}

#[test]
fn render_shows_suggestions_next_to_the_input() {
    let mut selector = Selector(CountryId::new(0));
    let mut app = app(&mut selector);
    let map = Map::load().expect("embedded map is valid");
    let mut input_map = Map::load().expect("embedded map is valid");
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).expect("test terminal starts");

    for character in "Sp".chars() {
        app.handle(
            EventAction::Input(input::InputAction::Insert(character)),
            &mut input_map,
        );
    }
    terminal
        .draw(|frame| app.render(frame, &map))
        .expect("frame renders");

    let input_row = terminal_row(&terminal, 24);
    let status_row = terminal_row(&terminal, 28);
    assert!(input_row.contains("Sp  Tab: Spain"));
    assert!(!status_row.contains("Tab:"));
}

#[test]
fn render_prioritizes_recoverable_messages_over_suggestions() {
    let mut selector = Selector(CountryId::new(0));
    let mut app = app(&mut selector);
    let map = Map::load().expect("embedded map is valid");
    let mut input_map = Map::load().expect("embedded map is valid");
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).expect("test terminal starts");

    for character in "Sp".chars() {
        app.handle(
            EventAction::Input(input::InputAction::Insert(character)),
            &mut input_map,
        );
    }
    app.message = Some("Unknown country.".to_owned());
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
    assert!(content.contains("Unknown country."));
    assert!(!content.contains("Tab: Spain"));
}

#[test]
fn hint_reveals_one_more_target_letter_without_an_attempt() {
    let mut selector = Selector(CountryId::new(1));
    let mut app = app(&mut selector);
    let mut map = Map::load().expect("embedded map is valid");

    for character in "/hint".chars() {
        app.handle(
            EventAction::Input(input::InputAction::Insert(character)),
            &mut map,
        );
    }
    app.handle(EventAction::Input(input::InputAction::Submit), &mut map);

    assert_eq!(app.message.as_deref(), Some("Hint: S____"));
    assert!(app.game.guesses().is_empty());
}

#[test]
fn surrender_reveals_the_target_and_allows_a_new_game() {
    let mut selector = Selector(CountryId::new(1));
    let mut app = app(&mut selector);
    let mut map = Map::load().expect("embedded map is valid");

    for character in "/surrender".chars() {
        app.handle(
            EventAction::Input(input::InputAction::Insert(character)),
            &mut map,
        );
    }
    app.handle(EventAction::Input(input::InputAction::Submit), &mut map);

    assert!(app.surrendered);
    assert!(
        app.message
            .as_deref()
            .is_some_and(|message| message.contains("Spain"))
    );
    assert!(app.game.guesses().is_empty());

    app.handle(
        EventAction::Input(input::InputAction::Insert('n')),
        &mut map,
    );
    assert!(!app.surrendered);
    assert!(app.message.is_none());
}

#[test]
fn status_keeps_a_surrender_message_visible_at_the_minimum_size() {
    let mut selector = Selector(CountryId::new(1));
    let mut app = app(&mut selector);
    let mut map = Map::load().expect("embedded map is valid");
    let mut terminal = Terminal::new(TestBackend::new(48, 20)).expect("test terminal starts");

    for character in "/surrender".chars() {
        app.handle(
            EventAction::Input(input::InputAction::Insert(character)),
            &mut map,
        );
    }
    app.handle(EventAction::Input(input::InputAction::Submit), &mut map);
    terminal
        .draw(|frame| app.render(frame, &map))
        .expect("frame renders");

    let status = format!(
        "{}{}",
        terminal_row(&terminal, 17),
        terminal_row(&terminal, 18)
    );
    assert!(status.contains("Surrendered. The target was Spain."));
    assert!(status.contains("Esc to quit."));
}

#[test]
fn accepted_guess_centers_the_map_without_changing_its_zoom() {
    let mut selector = Selector(CountryId::new(1));
    let mut app = app(&mut selector);
    let mut map = Map::load().expect("embedded map is valid");
    let (center_x, center_y, zoom) = map.camera();

    for character in "France".chars() {
        app.handle(
            EventAction::Input(input::InputAction::Insert(character)),
            &mut map,
        );
    }
    app.handle(EventAction::Input(input::InputAction::Submit), &mut map);

    let (new_center_x, new_center_y, new_zoom) = map.camera();
    assert_eq!(new_zoom, zoom);
    assert_ne!((new_center_x, new_center_y), (center_x, center_y));
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

fn terminal_row(terminal: &Terminal<TestBackend>, row: u16) -> String {
    (0..terminal.backend().buffer().area().width)
        .map(|column| terminal.backend().buffer()[(column, row)].symbol())
        .collect()
}
