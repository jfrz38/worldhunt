use crate::{
    application::{StartGame, StartGameError},
    domain::{
        CountryId, GuessInput,
        ports::{CountryCatalog, TargetSelector},
    },
};

struct Catalog(Vec<CountryId>);
impl CountryCatalog for Catalog {
    fn playable(&self) -> &[CountryId] {
        &self.0
    }
    fn name(&self, _: CountryId) -> Option<&str> {
        None
    }
    fn resolve(&self, _: &GuessInput) -> Option<CountryId> {
        None
    }
}
struct Selector(CountryId);
impl TargetSelector for Selector {
    fn select(&mut self, _: &[CountryId]) -> CountryId {
        self.0
    }
}

#[test]
fn starts_a_game_with_a_selected_eligible_country() {
    let catalog = Catalog(vec![CountryId::new(1), CountryId::new(2)]);
    let mut selector = Selector(CountryId::new(2));
    let mut use_case = StartGame::new(&catalog, &mut selector);

    assert_eq!(
        use_case.dispatch().expect("valid game").target(),
        CountryId::new(2)
    );
}

#[test]
fn rejects_empty_catalogs_and_invalid_selector_results() {
    let empty = Catalog(vec![]);
    let mut selector = Selector(CountryId::new(1));
    assert!(matches!(
        StartGame::new(&empty, &mut selector).dispatch(),
        Err(StartGameError::EmptyCatalog)
    ));

    let catalog = Catalog(vec![CountryId::new(1)]);
    let mut selector = Selector(CountryId::new(2));
    assert!(matches!(
        StartGame::new(&catalog, &mut selector).dispatch(),
        Err(StartGameError::InvalidTarget(target)) if target == CountryId::new(2)
    ));
}
