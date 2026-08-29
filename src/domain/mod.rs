//! Business concepts and rules, independent of application and infrastructure.

mod country;
mod game;
mod guess;
pub mod ports;
mod proximity;

pub use country::{CountryId, GuessInput};
pub use game::{Game, GameError, GameStatus};
pub use guess::{Guess, GuessClue};
pub use proximity::Proximity;
