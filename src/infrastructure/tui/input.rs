use crossterm::event::KeyCode;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum InputAction {
    Insert(char),
    Backspace,
    Complete,
    Submit,
}

pub(super) fn action_for(code: KeyCode) -> Option<InputAction> {
    match code {
        KeyCode::Char(character) if !character.is_control() => Some(InputAction::Insert(character)),
        KeyCode::Backspace => Some(InputAction::Backspace),
        KeyCode::Tab => Some(InputAction::Complete),
        KeyCode::Enter => Some(InputAction::Submit),
        _ => None,
    }
}

pub(super) fn apply(value: &mut String, action: InputAction) -> bool {
    match action {
        InputAction::Insert(character) => value.push(character),
        InputAction::Backspace => {
            if let Some((index, _)) = value.grapheme_indices(true).next_back() {
                value.truncate(index);
            }
        }
        InputAction::Complete => return false,
        InputAction::Submit => return false,
    }
    true
}

#[cfg(test)]
mod tests;
