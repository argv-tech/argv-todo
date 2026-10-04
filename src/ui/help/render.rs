use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarState},
};

use super::{
    super::{
        layout::centered,
        theme::{ACCENT, BG, MUTED, TEXT},
    },
    content,
};

pub(in crate::ui) fn draw_help(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let popup = centered(
        area,
        78.min(area.width.saturating_sub(u16::from(area.width > 2) * 2)),
        34.min(area.height.saturating_sub(u16::from(area.height > 2) * 2)),
    );
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .title(if popup.width >= 24 {
            " Keyboard shortcuts "
        } else {
            " Help "
        })
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .title_bottom(
            Line::from(if popup.width >= 18 {
                " Esc / ? close "
            } else {
                " Esc "
            })
            .centered(),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MUTED))
        .style(Style::default().bg(BG).fg(TEXT));
    let inner = block.inner(popup).inner(Margin {
        horizontal: u16::from(popup.width >= 24),
        vertical: 0,
    });
    let [content_area, footer] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(inner);
    frame.render_widget(block, popup);
    let lines = content::lines(content_area.width);
    app.help_view.set_viewport(lines.len(), content_area.height);
    if content_area.is_empty() {
        return;
    }
    frame.render_widget(
        Paragraph::new(lines).scroll((app.help_view.offset(), 0)),
        content_area,
    );
    let mut scrollbar = ScrollbarState::new(app.help_view.total_rows())
        .position(usize::from(app.help_view.offset()))
        .viewport_content_length(usize::from(content_area.height));
    frame.render_stateful_widget(
        Scrollbar::default()
            .begin_symbol(None)
            .end_symbol(None)
            .track_style(Style::default().fg(MUTED))
            .thumb_style(Style::default().fg(ACCENT)),
        Rect::new(popup.right() - 1, content_area.y, 1, content_area.height),
        &mut scrollbar,
    );
    let start = usize::from(app.help_view.offset()) + 1;
    let end = (start - 1 + usize::from(content_area.height)).min(app.help_view.total_rows());
    let position = format!("{start}-{end}/{}", app.help_view.total_rows());
    let position_width = if footer.width >= 26 {
        position.len() as u16 + 1
    } else {
        0
    };
    let [hint, progress] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(position_width)]).areas(footer);
    let hint_text = [
        "j/k scroll · PgUp/PgDn page · gg/G ends",
        "j/k scroll · PgUp/PgDn page",
        "j/k scroll",
        "j/k",
    ]
    .into_iter()
    .find(|text| Line::from(*text).width() <= usize::from(hint.width))
    .unwrap_or("");
    frame.render_widget(
        Paragraph::new(hint_text).style(Style::default().fg(MUTED)),
        hint,
    );
    frame.render_widget(
        Paragraph::new(Line::from(position).right_aligned()).style(Style::default().fg(MUTED)),
        progress,
    );
}
