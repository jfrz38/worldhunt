use super::{MINIMUM_HEIGHT, MINIMUM_WIDTH, map_area};
use ratatui::layout::Rect;

#[test]
fn rejects_terminal_below_minimum_size() {
    assert_eq!(
        map_area(Rect::new(0, 0, MINIMUM_WIDTH - 1, MINIMUM_HEIGHT)),
        None
    );
    assert_eq!(
        map_area(Rect::new(0, 0, MINIMUM_WIDTH, MINIMUM_HEIGHT - 1)),
        None
    );
}

#[test]
fn reserves_one_line_for_status() {
    assert_eq!(
        map_area(Rect::new(2, 3, 80, 24)),
        Some(Rect::new(2, 3, 80, 23))
    );
}
