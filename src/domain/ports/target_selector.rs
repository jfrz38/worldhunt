use crate::domain::CountryId;

pub trait TargetSelector {
    fn select(&mut self, eligible: &[CountryId]) -> CountryId;
}
