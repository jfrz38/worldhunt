use super::{CountryId, Guess, GuessClue, Proximity};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Playing,
    Won,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameError {
    DuplicateGuess,
    Completed,
}

/// A single local game and its legal guess transitions.
#[derive(Debug)]
pub struct Game {
    target: CountryId,
    guesses: Vec<Guess>,
    status: GameStatus,
}

impl Game {
    pub const fn new(target: CountryId) -> Self {
        Self {
            target,
            guesses: Vec::new(),
            status: GameStatus::Playing,
        }
    }

    pub const fn target(&self) -> CountryId {
        self.target
    }

    pub const fn status(&self) -> GameStatus {
        self.status
    }

    pub fn guesses(&self) -> &[Guess] {
        &self.guesses
    }

    pub fn has_guessed(&self, country: CountryId) -> bool {
        self.guesses.iter().any(|guess| guess.country() == country)
    }

    pub fn submit(&mut self, country: CountryId, proximity: Proximity) -> Result<Guess, GameError> {
        if self.status == GameStatus::Won {
            return Err(GameError::Completed);
        }
        if self.has_guessed(country) {
            return Err(GameError::DuplicateGuess);
        }

        let clue = if country == self.target {
            self.status = GameStatus::Won;
            GuessClue::Target
        } else if proximity.is_adjacent() {
            GuessClue::BordersTarget
        } else {
            GuessClue::DistanceKm(proximity.distance_km())
        };
        let guess = Guess::new(country, clue);
        self.guesses.push(guess);
        Ok(guess)
    }
}

#[cfg(test)]
mod tests;
