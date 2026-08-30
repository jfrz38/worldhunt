use super::{ColorMode, Theme};
use crate::domain::GuessClue;
use ratatui::style::Color;

#[test]
fn distance_bands_are_stable_at_their_boundaries() {
    let theme = Theme::new(ColorMode::Ansi256);

    assert_eq!(
        theme.guessed_land(GuessClue::DistanceKm(499)).bg,
        Some(Color::Indexed(203))
    );
    assert_eq!(
        theme.guessed_land(GuessClue::DistanceKm(500)).bg,
        Some(Color::Indexed(167))
    );
    assert_eq!(
        theme.guessed_land(GuessClue::DistanceKm(8_000)).bg,
        Some(Color::Indexed(52))
    );
}

#[test]
fn winning_style_is_not_a_distance_style() {
    let theme = Theme::new(ColorMode::TrueColor);

    assert_ne!(
        theme.guessed_land(GuessClue::Target).bg,
        theme.guessed_land(GuessClue::DistanceKm(1)).bg
    );
}
