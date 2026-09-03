use ratatui::layout::Rect;

pub(super) const MINIMUM_WIDTH: u16 = 20;
pub(super) const MINIMUM_HEIGHT: u16 = 8;

pub(super) fn map_area(area: Rect) -> Option<Rect> {
    if area.width < MINIMUM_WIDTH || area.height < MINIMUM_HEIGHT {
        return None;
    }

    // Reserve the final line for attribution and navigation status.
    Some(Rect::new(
        area.x,
        area.y,
        area.width,
        area.height.saturating_sub(1),
    ))
}

#[cfg(test)]
mod tests;
