use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use super::{EventAction, event_action};

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

#[test]
fn escape_and_control_c_quit() {
    assert_eq!(
        event_action(key(KeyCode::Esc, KeyModifiers::NONE)),
        EventAction::Quit
    );
    assert_eq!(
        event_action(key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        EventAction::Quit
    );
}

#[test]
fn resize_requests_a_redraw() {
    assert_eq!(event_action(Event::Resize(100, 40)), EventAction::Redraw);
}

#[test]
fn plain_q_does_not_quit() {
    assert_eq!(
        event_action(key(KeyCode::Char('q'), KeyModifiers::NONE)),
        EventAction::Wait
    );
}

#[test]
fn key_release_does_not_quit() {
    let event = Event::Key(KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Release,
        state: KeyEventState::NONE,
    });

    assert_eq!(event_action(event), EventAction::Wait);
}
