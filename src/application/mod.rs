//! Application orchestration, depending only on the domain layer.

mod start_game;
mod submit_guess;

pub use start_game::{StartGame, StartGameError};
pub use submit_guess::{SubmitGuess, SubmitGuessOutcome};
