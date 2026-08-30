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
            EventAction::Input(action) => self.handle_input(action),
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
                Constraint::Length(3),
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

    fn handle_input(&mut self, action: InputAction) {
        if matches!(action, InputAction::Insert('n' | 'N')) && self.game.status() == GameStatus::Won
        {
            self.start_new_game();
            return;
        }
        if !input::apply(&mut self.input, action) {
            self.submit();
        } else {
            self.message = None;
        }
    }

    fn submit(&mut self) {
        let outcome = SubmitGuess::new(self.catalog, self.proximity)
            .dispatch(&mut self.game, GuessInput::new(&self.input));
        self.message = match outcome {
            SubmitGuessOutcome::Accepted(_) => {
                self.input.clear();
                None
            }
            SubmitGuessOutcome::Won(_) => {
                self.input.clear();
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
        } else {
            " Guess a country "
        };
        let content = if self.game.status() == GameStatus::Won {
            format!(
                "Target: {}   N: new game   Esc: quit",
                self.country_name(self.game.target())
            )
        } else {
            self.input.clone()
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
            .unwrap_or("Enter: submit   Arrows: pan   +/-: zoom   Esc: quit");
        frame.render_widget(
            Paragraph::new(Span::styled(
                message,
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
}

#[cfg(test)]
mod tests;
