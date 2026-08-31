use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseEvent, MouseEventKind,
};

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
        EventAction::Input(super::input::InputAction::Insert('q'))
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

#[test]
fn mouse_wheel_zooms_without_panning() {
    let up = Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    });
    let down = Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 0,
        row: 0,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(event_action(up), EventAction::Zoom(1));
    assert_eq!(event_action(down), EventAction::Zoom(-1));
}
