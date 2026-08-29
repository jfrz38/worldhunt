use crate::domain::{
    Game, GameError, GameStatus, Guess, GuessInput, Proximity,
    ports::{CountryCatalog, CountryProximity},
};

pub struct SubmitGuess<'a, Catalog, ProximityPort> {
    catalog: &'a Catalog,
    proximity: &'a ProximityPort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmitGuessOutcome {
    Accepted(Guess),
    Won(Guess),
    EmptyInput,
    UnknownCountry,
    DuplicateGuess,
    GameCompleted,
    ProximityUnavailable,
}

impl<'a, Catalog, ProximityPort> SubmitGuess<'a, Catalog, ProximityPort>
where
    Catalog: CountryCatalog,
    ProximityPort: CountryProximity,
{
    pub fn new(catalog: &'a Catalog, proximity: &'a ProximityPort) -> Self {
        Self { catalog, proximity }
    }

    pub fn dispatch(&self, game: &mut Game, input: GuessInput) -> SubmitGuessOutcome {
        if input.is_empty() {
            return SubmitGuessOutcome::EmptyInput;
        }
        if game.status() == GameStatus::Won {
            return SubmitGuessOutcome::GameCompleted;
        }
        let Some(country) = self.catalog.resolve(&input) else {
            return SubmitGuessOutcome::UnknownCountry;
        };
        if game.has_guessed(country) {
            return SubmitGuessOutcome::DuplicateGuess;
        }
        let proximity = if country == game.target() {
            Proximity::new(0, false)
        } else {
            let Some(proximity) = self.proximity.between(country, game.target()) else {
                return SubmitGuessOutcome::ProximityUnavailable;
            };
            proximity
        };
        match game.submit(country, proximity) {
            Ok(guess) if guess.country() == game.target() => SubmitGuessOutcome::Won(guess),
            Ok(guess) => SubmitGuessOutcome::Accepted(guess),
            Err(GameError::DuplicateGuess) => SubmitGuessOutcome::DuplicateGuess,
            Err(GameError::Completed) => SubmitGuessOutcome::GameCompleted,
        }
    }
}

#[cfg(test)]
mod tests;
