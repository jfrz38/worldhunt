use rand::{SeedableRng, prelude::IndexedRandom, rngs::StdRng};

use crate::domain::{CountryId, ports::TargetSelector};

/// Random target selection kept outside the game core.
pub struct RandomTargetSelector(StdRng);

impl Default for RandomTargetSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomTargetSelector {
    pub fn new() -> Self {
        Self(StdRng::from_os_rng())
    }

    pub fn seeded(seed: u64) -> Self {
        Self(StdRng::seed_from_u64(seed))
    }
}

impl TargetSelector for RandomTargetSelector {
    fn select(&mut self, eligible: &[CountryId]) -> CountryId {
        *eligible
            .choose(&mut self.0)
            .expect("target selection requires an eligible country")
    }
}

#[cfg(test)]
mod tests;
