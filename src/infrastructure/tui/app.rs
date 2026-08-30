use std::io;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    application::{StartGame, SubmitGuess, SubmitGuessOutcome},
    domain::{
        CountryId, Game, GameStatus, GuessClue, GuessInput,
        ports::{CountryCatalog, CountryProximity, TargetSelector},
    },
};

use super::{
    EventAction,
    input::{self, InputAction},
    map::Map,
};

const MINIMUM_WIDTH: u16 = 48;
const MINIMUM_HEIGHT: u16 = 20;

pub(super) struct TuiApp<'a, Catalog, ProximityPort, Selector> {
    catalog: &'a Catalog,
    proximity: &'a ProximityPort,
    selector: &'a mut Selector,
    game: Game,
    input: String,
    message: Option<String>,
    hint_count: usize,
    surrendered: bool,
    should_quit: bool,
}

impl<'a, Catalog, ProximityPort, Selector> TuiApp<'a, Catalog, ProximityPort, Selector>
where
    Catalog: CountryCatalog,
    ProximityPort: CountryProximity,
    Selector: TargetSelector,
{
    pub(super) fn new(
        catalog: &'a Catalog,
        proximity: &'a ProximityPort,
        selector: &'a mut Selector,
    ) -> io::Result<Self> {
        let game = StartGame::new(catalog, selector)
            .dispatch()
            .map_err(|error| io::Error::other(format!("cannot start game: {error:?}")))?;
        Ok(Self {
            catalog,
            proximity,
            selector,
            game,
            input: String::new(),
            message: None,
            hint_count: 0,
            surrendered: false,
            should_quit: false,
        })
    }

    pub(super) fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub(super) fn handle(&mut self, action: EventAction, map: &mut Map) {
        match action {
            EventAction::Quit => self.should_quit = true,
            EventAction::Zoom(steps) => {
                for _ in 0..steps.unsigned_abs() {
                    if steps.is_positive() {
                        map.zoom_in();
                    } else {
                        map.zoom_out();
                    }
                }
            }
            EventAction::Pan(horizontal, vertical) => map.pan(horizontal, vertical),
            EventAction::Input(action) => self.handle_input(action, map),
            EventAction::Redraw | EventAction::Wait => {}
        }
    }

    pub(super) fn render(&self, frame: &mut Frame, map: &Map) {
        let area = frame.area();
        if area.width < MINIMUM_WIDTH || area.height < MINIMUM_HEIGHT {
            frame.render_widget(
                Paragraph::new("Resize terminal to at least 48 x 20 to play WorldHunt.")
                    .block(Block::default().borders(Borders::ALL).title(" WorldHunt "))
                    .wrap(Wrap { trim: true }),
                area,
            );
            return;
        }

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Length(4),
            ])
            .split(area);
        let content = if area.width >= 90 {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
                .split(sections[0])
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(8), Constraint::Length(6)])
                .split(sections[0])
        };

        let map_area = Block::default()
            .borders(Borders::ALL)
            .title(" WorldHunt ")
            .inner(content[0]);
        frame.render_widget(
            Block::default().borders(Borders::ALL).title(" WorldHunt "),
            content[0],
        );
        map.render_with_guesses(
            map_area,
            frame.buffer_mut(),
            self.game.guesses(),
            super::theme::Theme::from_environment(),
        );
        self.render_history(frame, content[1]);
        self.render_input(frame, sections[1]);
        self.render_message(frame, sections[2]);
    }

    fn handle_input(&mut self, action: InputAction, map: &mut Map) {
        if matches!(action, InputAction::Insert('n' | 'N')) && self.game_is_finished() {
            self.start_new_game();
            return;
        }
        if self.game_is_finished() {
            return;
        }
        if action == InputAction::Complete {
            if let Some(country) = self.suggestions().first().copied()
                && let Some(name) = self.catalog.name(country)
            {
                self.input = name.to_owned();
                self.message = None;
            }
        } else if !input::apply(&mut self.input, action) {
            self.submit(map);
        } else {
            self.message = None;
        }
    }

    fn submit(&mut self, map: &mut Map) {
        match self.input.trim().to_ascii_lowercase().as_str() {
            "/surrender" => {
                self.surrendered = true;
                self.input.clear();
                map.center_on(self.game.target());
                self.message = Some(format!(
                    "Surrendered. The target was {}. Press N for a new game or Esc to quit.",
                    self.country_name(self.game.target())
                ));
                return;
            }
            "/hint" => {
                self.hint_count += 1;
                self.input.clear();
                self.message = Some(format!("Hint: {}", self.hint_text()));
                return;
            }
            _ => {}
        }
        let outcome = SubmitGuess::new(self.catalog, self.proximity)
            .dispatch(&mut self.game, GuessInput::new(&self.input));
        self.message = match outcome {
            SubmitGuessOutcome::Accepted(guess) => {
                self.input.clear();
                map.center_on(guess.country());
                None
            }
            SubmitGuessOutcome::Won(guess) => {
                self.input.clear();
                map.center_on(guess.country());
                Some(format!(
                    "You found {} in {} attempts. Press N for a new game or Esc to quit.",
                    self.country_name(self.game.target()),
                    self.game.guesses().len()
                ))
            }
            SubmitGuessOutcome::EmptyInput => Some("Enter a country name.".to_owned()),
            SubmitGuessOutcome::UnknownCountry => {
                Some("Unknown country. Try a canonical name or known alias.".to_owned())
            }
            SubmitGuessOutcome::DuplicateGuess => {
                Some("That country has already been guessed.".to_owned())
            }
            SubmitGuessOutcome::GameCompleted => {
                Some("Press N for a new game or Esc to quit.".to_owned())
            }
            SubmitGuessOutcome::ProximityUnavailable => {
                Some("Distance data is unavailable for that country.".to_owned())
            }
        };
    }

    fn start_new_game(&mut self) {
        match StartGame::new(self.catalog, self.selector).dispatch() {
            Ok(game) => {
                self.game = game;
                self.input.clear();
                self.message = None;
                self.hint_count = 0;
                self.surrendered = false;
            }
            Err(error) => self.message = Some(format!("Cannot start a new game: {error:?}")),
        }
    }

    fn render_history(&self, frame: &mut Frame, area: Rect) {
        let available = usize::from(area.height.saturating_sub(2));
        let start = self.game.guesses().len().saturating_sub(available);
        let lines = self.game.guesses()[start..]
            .iter()
            .map(|guess| {
                let clue = match guess.clue() {
                    GuessClue::DistanceKm(distance) => format!("{distance} km"),
                    GuessClue::BordersTarget => "Borders target".to_owned(),
                    GuessClue::Target => "Target".to_owned(),
                };
                Line::from(format!("{}  {clue}", self.country_name(guess.country())))
            })
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" Attempts ({}) ", self.game.guesses().len())),
                )
                .wrap(Wrap { trim: true }),
            area,
        );
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let title = if self.game.status() == GameStatus::Won {
            " Victory "
        } else if self.surrendered {
            " Surrendered "
        } else {
            " Guess a country "
        };
        let content = if self.game_is_finished() {
            format!(
                "Target: {}   N: new game   Esc: quit",
                self.country_name(self.game.target())
            )
        } else {
            let input_width = usize::from(area.width.saturating_sub(2));
            let suggestion_width = input_width
                .saturating_sub(self.input.chars().count())
                .saturating_sub(2);
            let suggestions = if self.message.is_none() {
                self.suggestion_text(suggestion_width as u16)
            } else {
                String::new()
            };
            if suggestions.is_empty() {
                self.input.clone()
            } else {
                format!("{}  {suggestions}", self.input)
            }
        };
        frame.render_widget(
            Paragraph::new(content).block(Block::default().borders(Borders::ALL).title(title)),
            area,
        );
    }

    fn render_message(&self, frame: &mut Frame, area: Rect) {
        let message = self
            .message
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| "Enter: submit   Arrows: pan   +/-: zoom   Esc: quit".to_owned());
        frame.render_widget(
            Paragraph::new(Span::styled(
                &message,
                Style::default().add_modifier(Modifier::DIM),
            ))
            .block(Block::default().borders(Borders::ALL).title(" Status "))
            .wrap(Wrap { trim: true }),
            area,
        );
    }

    fn country_name(&self, country: CountryId) -> &str {
        self.catalog.name(country).unwrap_or("Unknown country")
    }

    fn suggestions(&self) -> Vec<CountryId> {
        self.catalog.search(&GuessInput::new(&self.input), 5)
    }

    fn suggestion_text(&self, width: u16) -> String {
        let mut text = String::from("Tab: ");
        for country in self.suggestions() {
            let Some(name) = self.catalog.name(country) else {
                continue;
            };
            let separator = if text.ends_with(' ') { "" } else { "  " };
            if text.chars().count() + separator.chars().count() + name.chars().count()
                > usize::from(width)
            {
                break;
            }
            text.push_str(separator);
            text.push_str(name);
        }
        if text == "Tab: " { String::new() } else { text }
    }

    fn game_is_finished(&self) -> bool {
        self.game.status() == GameStatus::Won || self.surrendered
    }

    fn hint_text(&self) -> String {
        let mut remaining = self.hint_count;
        self.country_name(self.game.target())
            .chars()
            .map(|character| {
                if !character.is_alphabetic() {
                    character
                } else if remaining == 0 {
                    '_'
                } else {
                    remaining -= 1;
                    character
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
