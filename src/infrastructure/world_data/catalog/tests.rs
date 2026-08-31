use super::{RuntimeCountryCatalog, normalize_name};
use crate::domain::{GuessInput, ports::CountryCatalog};

#[test]
fn normalizes_diacritics_whitespace_and_punctuation() {
    assert_eq!(normalize_name("  Cote d'Ivoire "), "cote d ivoire");
}

#[test]
fn resolves_canonical_names_and_aliases() {
    let catalog = RuntimeCountryCatalog::load(196).expect("embedded catalog is valid");
    assert_eq!(catalog.playable().len(), 196);
    assert_eq!(
        catalog.resolve(&GuessInput::new("  uSa  ")),
        catalog.resolve(&GuessInput::new("United States"))
    );
    assert_eq!(
        catalog.resolve(&GuessInput::new("Côte d’Ivoire")),
        catalog.resolve(&GuessInput::new("Cote d'Ivoire"))
    );
}

#[test]
fn searches_normalized_prefixes_and_returns_canonical_matches() {
    let catalog = RuntimeCountryCatalog::load(196).expect("embedded catalog is valid");

    let results = catalog.search(&GuessInput::new("sp"), 5);

    assert!(
        results
            .iter()
            .any(|country| catalog.name(*country) == Some("Spain"))
    );
    assert!(catalog.search(&GuessInput::new("s"), 5).is_empty());
}

#[test]
fn suggests_the_shortest_matching_alias_for_moldova() {
    let catalog = RuntimeCountryCatalog::load(196).expect("embedded catalog is valid");

    assert!(
        catalog
            .suggestions(&GuessInput::new("Mo"), 5)
            .iter()
            .any(|suggestion| suggestion.completion == "Moldova")
    );
}
