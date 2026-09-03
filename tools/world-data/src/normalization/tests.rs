use super::normalize_name;

#[test]
fn normalizes_case_whitespace_punctuation_and_diacritics() {
    assert_eq!(normalize_name("  Cote--d'Ivoire  "), "cote d ivoire");
    assert_eq!(normalize_name("TÜRKIYE"), "turkiye");
    assert_eq!(normalize_name("United\t States\n"), "united states");
}
