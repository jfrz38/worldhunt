use crossterm::event::KeyCode;

use super::{InputAction, action_for, apply};

#[test]
fn inserts_printable_characters_and_removes_one_grapheme() {
    let mut input = String::from("Cote\u{301}");

    assert!(apply(&mut input, InputAction::Insert(' ')));
    assert!(apply(&mut input, InputAction::Backspace));
    assert!(apply(&mut input, InputAction::Backspace));

    assert_eq!(input, "Cot");
}

#[test]
fn enter_is_a_submission_without_editing_input() {
    let mut input = String::from("France");

    assert!(!apply(&mut input, InputAction::Submit));
    assert_eq!(input, "France");
    assert_eq!(action_for(KeyCode::Enter), Some(InputAction::Submit));
}
