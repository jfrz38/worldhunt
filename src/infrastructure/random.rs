use rand::{SeedableRng, prelude::IndexedRandom, rngs::StdRng};

use crate::domain::{CountryId, ports::TargetSelector};

/// Random target selection kept outside the game core.
pub struct RandomTargetSelector(StdRng);

impl RandomTargetSelector {
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
mod tests {
    use crate::{
        domain::{CountryId, ports::TargetSelector},
        infrastructure::random::RandomTargetSelector,
    };

    #[test]
    fn seeded_selectors_produce_the_same_sequence() {
        let eligible = [CountryId::new(1), CountryId::new(2), CountryId::new(3)];
        let mut first = RandomTargetSelector::seeded(7);
        let mut second = RandomTargetSelector::seeded(7);

        let first_sequence = (0..10).map(|_| first.select(&eligible)).collect::<Vec<_>>();
        let second_sequence = (0..10)
            .map(|_| second.select(&eligible))
            .collect::<Vec<_>>();
        assert_eq!(first_sequence, second_sequence);
    }
}
