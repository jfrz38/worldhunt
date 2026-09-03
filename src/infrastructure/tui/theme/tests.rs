use super::{ColorMode, Theme};
use crate::domain::GuessClue;
use ratatui::style::{Color, Modifier};
use std::sync::Mutex;

static ENVIRONMENT_LOCK: Mutex<()> = Mutex::new(());

struct EnvironmentGuard {
    color: Option<std::ffi::OsString>,
    no_color: Option<std::ffi::OsString>,
}

impl EnvironmentGuard {
    fn clear() -> Self {
        let guard = Self {
            color: std::env::var_os("WORLDHUNT_COLOR"),
            no_color: std::env::var_os("NO_COLOR"),
        };
        unsafe {
            std::env::remove_var("WORLDHUNT_COLOR");
            std::env::remove_var("NO_COLOR");
        }
        guard
    }
}

impl Drop for EnvironmentGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.color {
                Some(value) => std::env::set_var("WORLDHUNT_COLOR", value),
                None => std::env::remove_var("WORLDHUNT_COLOR"),
            }
            match &self.no_color {
                Some(value) => std::env::set_var("NO_COLOR", value),
                None => std::env::remove_var("NO_COLOR"),
            }
        }
    }
}

fn with_color_environment(test: impl FnOnce()) {
    let _lock = ENVIRONMENT_LOCK.lock().expect("environment lock");
    let _environment = EnvironmentGuard::clear();
    test();
}

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

#[test]
fn environment_override_has_documented_precedence() {
    with_color_environment(|| {
        assert_eq!(Theme::from_environment().mode, ColorMode::TrueColor);
        unsafe { std::env::set_var("NO_COLOR", "1") };
        assert_eq!(Theme::from_environment().mode, ColorMode::Monochrome);
        unsafe { std::env::set_var("WORLDHUNT_COLOR", "ansi256") };
        assert_eq!(Theme::from_environment().mode, ColorMode::Ansi256);
        unsafe { std::env::set_var("WORLDHUNT_COLOR", "mono") };
        assert_eq!(Theme::from_environment().mode, ColorMode::Monochrome);
        unsafe { std::env::set_var("WORLDHUNT_COLOR", "unknown") };
        assert_eq!(Theme::from_environment().mode, ColorMode::Monochrome);
    });
}

#[test]
fn monochrome_clues_remain_distinguishable_without_color() {
    let theme = Theme::new(ColorMode::Monochrome);

    assert!(
        theme
            .guessed_land(GuessClue::Target)
            .add_modifier
            .contains(Modifier::BOLD)
    );
    assert!(
        theme
            .guessed_land(GuessClue::BordersTarget)
            .add_modifier
            .contains(Modifier::UNDERLINED)
    );
    assert!(
        theme
            .guessed_land(GuessClue::DistanceKm(1))
            .add_modifier
            .contains(Modifier::REVERSED)
    );
}
