use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

pub(crate) fn normalize_name(name: &str) -> String {
    let mut normalized = String::new();
    let mut needs_space = false;

    for character in name.nfkd().flat_map(char::to_lowercase) {
        if is_combining_mark(character) {
            continue;
        }

        if character.is_alphanumeric() {
            if needs_space && !normalized.is_empty() {
                normalized.push(' ');
            }
            normalized.push(character);
            needs_space = false;
        } else {
            needs_space = true;
        }
    }

    normalized
}

#[cfg(test)]
mod tests;
