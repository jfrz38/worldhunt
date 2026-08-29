use std::cell::Cell;

use crate::{
    application::{SubmitGuess, SubmitGuessOutcome},
    domain::{
        CountryId, Game, GuessClue, GuessInput, Proximity,
        ports::{CountryCatalog, CountryProximity},
    },
};

struct Catalog;
impl CountryCatalog for Catalog {
    fn playable(&self) -> &[CountryId] {
        &[]
    }
    fn resolve(&self, input: &GuessInput) -> Option<CountryId> {
        (input.as_str() == "France").then_some(CountryId::new(1))
    }
}
struct ProximityPort(Cell<u8>);
impl CountryProximity for ProximityPort {
    fn between(&self, _: CountryId, _: CountryId) -> Option<Proximity> {
        self.0.set(self.0.get() + 1);
        Some(Proximity::new(0, true))
    }
}

struct MissingProximityPort;
impl CountryProximity for MissingProximityPort {
    fn between(&self, _: CountryId, _: CountryId) -> Option<Proximity> {
        None
    }
}

#[test]
fn invalid_inputs_do_not_query_proximity_or_mutate_the_game() {
    let proximity = ProximityPort(Cell::new(0));
    let use_case = SubmitGuess::new(&Catalog, &proximity);
    let mut game = Game::new(CountryId::new(2));

    assert_eq!(
        use_case.dispatch(&mut game, GuessInput::new("  ")),
        SubmitGuessOutcome::EmptyInput
    );
    assert_eq!(
        use_case.dispatch(&mut game, GuessInput::new("Unknown")),
        SubmitGuessOutcome::UnknownCountry
    );
    assert_eq!(proximity.0.get(), 0);
    assert!(game.guesses().is_empty());
}

#[test]
fn records_an_accepted_guess_and_a_win() {
    let proximity = ProximityPort(Cell::new(0));
    let use_case = SubmitGuess::new(&Catalog, &proximity);
    let mut game = Game::new(CountryId::new(2));
    assert_eq!(
        use_case.dispatch(&mut game, GuessInput::new("France")),
        SubmitGuessOutcome::Accepted(crate::domain::Guess::new(
            CountryId::new(1),
            GuessClue::BordersTarget
        ))
    );

    let mut winning_game = Game::new(CountryId::new(1));
    assert!(matches!(
        use_case.dispatch(&mut winning_game, GuessInput::new("France")),
        SubmitGuessOutcome::Won(_)
    ));
    assert_eq!(
        use_case.dispatch(&mut winning_game, GuessInput::new("France")),
        SubmitGuessOutcome::GameCompleted
    );
    assert_eq!(proximity.0.get(), 1);
}

#[test]
fn repeated_guesses_do_not_query_proximity_again() {
    let proximity = ProximityPort(Cell::new(0));
    let use_case = SubmitGuess::new(&Catalog, &proximity);
    let mut game = Game::new(CountryId::new(2));

    assert!(matches!(
        use_case.dispatch(&mut game, GuessInput::new("France")),
        SubmitGuessOutcome::Accepted(_)
    ));
    assert_eq!(
        use_case.dispatch(&mut game, GuessInput::new("France")),
        SubmitGuessOutcome::DuplicateGuess
    );
    assert_eq!(proximity.0.get(), 1);
}

#[test]
fn unavailable_proximity_does_not_mutate_the_game() {
    let use_case = SubmitGuess::new(&Catalog, &MissingProximityPort);
    let mut game = Game::new(CountryId::new(2));

    assert_eq!(
        use_case.dispatch(&mut game, GuessInput::new("France")),
        SubmitGuessOutcome::ProximityUnavailable
    );
    assert!(game.guesses().is_empty());
}
