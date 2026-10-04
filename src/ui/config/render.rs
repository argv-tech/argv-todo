use super::super::theme::{ACCENT, ERROR, MUTED};
use super::{details::draw_details, form::draw_form};
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

pub(in crate::ui) fn draw_config(frame: &mut Frame, app: &App, area: Rect) {
    let branding_height = super::super::branding::height(area);
    let spacious = area.height >= 17;
    let [logo, _, title, _, body, status, footer] = Layout::vertical([
        Constraint::Length(branding_height),
        Constraint::Length(u16::from(spacious)),
        Constraint::Length(1),
        Constraint::Length(u16::from(spacious || area.width >= 70)),
        Constraint::Min(3),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    super::super::branding::draw(frame, logo);
    frame.render_widget(
        Paragraph::new("configuration").style(Style::default().add_modifier(Modifier::BOLD)),
        title,
    );
    if area.width >= 70 {
        let [form, _, details] = Layout::horizontal([
            Constraint::Percentage(42),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .areas(body);
        draw_form(frame, app, form);
        let divider = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(MUTED));
        let content = divider.inner(details).inner(Margin {
            horizontal: 1,
            vertical: 0,
        });
        frame.render_widget(divider, details);
        draw_details(frame, app, content, false);
    } else {
        let [form, details] = Layout::vertical([
            Constraint::Length(body.height.saturating_sub(3).clamp(3, 10)),
            Constraint::Min(0),
        ])
        .areas(body);
        draw_form(frame, app, form);
        draw_details(frame, app, details, true);
    }
    frame.render_widget(
        Paragraph::new(app.status.as_str()).style(Style::default().fg(if app.error {
            ERROR
        } else {
            ACCENT
        })),
        status,
    );
    let hint = if app.vim.input().is_some() {
        super::super::chrome::draw_footer(frame, app, footer);
        return;
    } else if !app.show_hints() {
        return;
    } else if area.width < 45 {
        "j/k setting · Enter change · Esc"
    } else {
        "j/k setting · Enter/e change · Esc tasks · q quit"
    };
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().fg(MUTED)),
        footer,
    );
}
