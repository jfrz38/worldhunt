use super::RandomTargetSelector;
use crate::domain::{CountryId, ports::TargetSelector};

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
