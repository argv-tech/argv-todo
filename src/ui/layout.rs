use ratatui::layout::{Margin, Rect};

pub(super) fn workspace(area: Rect) -> Rect {
    area.inner(Margin {
        horizontal: if area.width >= 60 { 2 } else { 1 },
        vertical: 1,
    })
}
