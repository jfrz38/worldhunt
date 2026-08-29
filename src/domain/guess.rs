use super::CountryId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuessClue {
    DistanceKm(u16),
    BordersTarget,
    Target,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guess {
    country: CountryId,
    clue: GuessClue,
}

impl Guess {
    pub(crate) const fn new(country: CountryId, clue: GuessClue) -> Self {
        Self { country, clue }
    }

    pub const fn country(self) -> CountryId {
        self.country
    }

    pub const fn clue(self) -> GuessClue {
        self.clue
    }
}
