use super::{Game, GameError, GameStatus};
use crate::domain::{CountryId, GuessClue, Proximity};

#[test]
fn records_distance_and_rejects_repeated_guesses() {
    let mut game = Game::new(CountryId::new(2));

    assert_eq!(
        game.submit(CountryId::new(1), Proximity::new(123, false)),
        Ok(crate::domain::Guess::new(
            CountryId::new(1),
            GuessClue::DistanceKm(123)
        ))
    );
    assert_eq!(
        game.submit(CountryId::new(1), Proximity::new(123, false)),
        Err(GameError::DuplicateGuess)
    );
    assert_eq!(game.guesses().len(), 1);
}

#[test]
fn records_border_and_target_clues_and_completes_the_game() {
    let mut game = Game::new(CountryId::new(2));

    assert_eq!(
        game.submit(CountryId::new(1), Proximity::new(0, true))
            .expect("border guess is accepted")
            .clue(),
        GuessClue::BordersTarget
    );
    assert_eq!(
        game.submit(CountryId::new(2), Proximity::new(0, false))
            .expect("target guess is accepted")
            .clue(),
        GuessClue::Target
    );
    assert_eq!(game.status(), GameStatus::Won);
    assert_eq!(
        game.submit(CountryId::new(0), Proximity::new(5, false)),
        Err(GameError::Completed)
    );
}
