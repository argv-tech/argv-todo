use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarState},
};

use super::{
    super::{
        branding,
        layout::workspace,
        theme::{ACCENT, MUTED},
    },
    chrome::{draw_footer, draw_sections},
    content,
};

pub(in crate::ui) fn draw_help(frame: &mut Frame, app: &mut App) {
    let area = workspace(frame.area());
    let spacious = area.height >= 17;
    let [brand, _, title, _, body, footer] = Layout::vertical([
        Constraint::Length(branding::height(area)),
        Constraint::Length(u16::from(spacious)),
        Constraint::Length(1),
        Constraint::Length(u16::from(spacious)),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .areas(area);
    branding::draw(frame, brand);
    frame.render_widget(
        Paragraph::new(if title.width >= 18 {
            "Keyboard shortcuts"
        } else {
            "Help"
        })
        .style(Style::default().add_modifier(Modifier::BOLD)),
        title,
    );
    let (sections, content_area) = if body.width >= 70 && body.height >= 9 {
        let [sections, _, details] = Layout::horizontal([
            Constraint::Length(30),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .areas(body);
        let divider = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(MUTED));
        let inner = divider.inner(details).inner(Margin {
            horizontal: 1,
            vertical: 0,
        });
        frame.render_widget(divider, details);
        (Some(sections), inner)
    } else {
        (None, body)
    };
    let [shortcuts, scrollbar_area] = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(u16::from(content_area.width > 1)),
    ])
    .areas(content_area);
    let document = content::document(shortcuts.width);
    app.help_view
        .set_sections(sections.map(|_| document.sections.len()));
    let range = if sections.is_some() {
        document.section_range(app.help_view.selected_section())
    } else {
        0..document.lines.len()
    };
    app.help_view.set_viewport(range.len(), shortcuts.height);
    if let Some(area) = sections {
        draw_sections(frame, &document, app, area);
    }
    if !shortcuts.is_empty() {
        frame.render_widget(
            Paragraph::new(
                document
                    .lines
                    .into_iter()
                    .skip(range.start)
                    .take(range.len())
                    .collect::<Vec<_>>(),
            )
            .scroll((app.help_view.offset(), 0)),
            shortcuts,
        );
        let mut scrollbar = ScrollbarState::new(app.help_view.total_rows())
            .position(usize::from(app.help_view.offset()))
            .viewport_content_length(usize::from(shortcuts.height));
        frame.render_stateful_widget(
            Scrollbar::default()
                .begin_symbol(None)
                .end_symbol(None)
                .track_style(Style::default().fg(MUTED))
                .thumb_style(Style::default().fg(ACCENT)),
            scrollbar_area,
            &mut scrollbar,
        );
    }
    draw_footer(frame, app, shortcuts.height, footer);
}
