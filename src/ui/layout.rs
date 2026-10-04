use ratatui::layout::{Margin, Rect};

pub(super) fn workspace(area: Rect) -> Rect {
    area.inner(Margin {
        horizontal: if area.width >= 60 { 2 } else { 1 },
        vertical: 1,
    })
}

pub(super) fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}
