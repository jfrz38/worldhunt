use ratatui::style::{Color, Modifier, Style};

use crate::domain::GuessClue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ColorMode {
    TrueColor,
    Ansi256,
    Monochrome,
}

#[derive(Clone, Copy)]
pub(super) struct Theme {
    mode: ColorMode,
}

impl Theme {
    pub(super) fn from_environment() -> Self {
        let requested = std::env::var("WORLDHUNT_COLOR").ok();
        let mode = match requested.as_deref() {
            Some("truecolor") => ColorMode::TrueColor,
            Some("ansi256") => ColorMode::Ansi256,
            Some("mono") => ColorMode::Monochrome,
            _ if std::env::var_os("NO_COLOR").is_some() => ColorMode::Monochrome,
            _ => ColorMode::TrueColor,
        };
        Self { mode }
    }

    #[cfg(test)]
    pub(super) const fn new(mode: ColorMode) -> Self {
        Self { mode }
    }

    pub(super) const fn water(self) -> Color {
        match self.mode {
            ColorMode::TrueColor => Color::Rgb(8, 24, 42),
            ColorMode::Ansi256 => Color::Indexed(17),
            ColorMode::Monochrome => Color::Black,
        }
    }

    pub(super) const fn border(self) -> Color {
        match self.mode {
            ColorMode::TrueColor => Color::Rgb(37, 48, 56),
            ColorMode::Ansi256 => Color::Indexed(238),
            ColorMode::Monochrome => Color::Gray,
        }
    }

    pub(super) const fn land(self) -> Color {
        match self.mode {
            ColorMode::TrueColor => Color::Rgb(93, 105, 112),
            ColorMode::Ansi256 => Color::Indexed(245),
            ColorMode::Monochrome => Color::DarkGray,
        }
    }

    pub(super) const fn neutral_land(self) -> Color {
        match self.mode {
            ColorMode::TrueColor => Color::Rgb(69, 78, 83),
            ColorMode::Ansi256 => Color::Indexed(240),
            ColorMode::Monochrome => Color::Black,
        }
    }

    pub(super) fn guessed_land(self, clue: GuessClue) -> Style {
        let style = match clue {
            GuessClue::Target => Style::new().bg(match self.mode {
                ColorMode::TrueColor => Color::Rgb(42, 152, 88),
                ColorMode::Ansi256 => Color::Indexed(35),
                ColorMode::Monochrome => Color::White,
            }),
            GuessClue::BordersTarget => Style::new().bg(match self.mode {
                ColorMode::TrueColor => Color::Rgb(229, 107, 38),
                ColorMode::Ansi256 => Color::Indexed(208),
                ColorMode::Monochrome => Color::Gray,
            }),
            GuessClue::DistanceKm(distance) => Style::new().bg(self.distance(distance)),
        };
        if self.mode == ColorMode::Monochrome {
            style.add_modifier(match clue {
                GuessClue::Target => Modifier::BOLD,
                GuessClue::BordersTarget => Modifier::UNDERLINED,
                GuessClue::DistanceKm(distance) if distance < 500 => Modifier::REVERSED,
                _ => Modifier::empty(),
            })
        } else {
            style
        }
    }

    const fn distance(self, distance: u16) -> Color {
        match self.mode {
            ColorMode::TrueColor => match distance {
                0..500 => Color::Rgb(255, 73, 66),
                500..1000 => Color::Rgb(221, 57, 53),
                1000..2000 => Color::Rgb(181, 48, 48),
                2000..4000 => Color::Rgb(136, 44, 48),
                4000..8000 => Color::Rgb(94, 40, 45),
                _ => Color::Rgb(63, 38, 43),
            },
            ColorMode::Ansi256 => Color::Indexed(match distance {
                0..500 => 203,
                500..1000 => 167,
                1000..2000 => 131,
                2000..4000 => 124,
                4000..8000 => 88,
                _ => 52,
            }),
            ColorMode::Monochrome => Color::DarkGray,
        }
    }
}

#[cfg(test)]
mod tests;
