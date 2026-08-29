/// Stable identifier assigned by the playable-country catalog order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CountryId(u16);

impl CountryId {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

/// Text supplied by a player for country resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuessInput(String);

impl GuessInput {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}
