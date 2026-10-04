use super::App;
use crate::vim_motion::{Motion, VimAction};

#[derive(Default)]
pub(crate) struct HelpView {
    offset: u16,
    height: u16,
    total_rows: usize,
}

impl HelpView {
    pub(crate) fn offset(&self) -> u16 {
        self.offset
    }

    pub(crate) fn total_rows(&self) -> usize {
        self.total_rows
    }

    pub(crate) fn set_viewport(&mut self, total_rows: usize, height: u16) {
        self.total_rows = total_rows;
        self.height = height;
        self.offset = self.offset.min(self.max_offset());
    }

    fn max_offset(&self) -> u16 {
        self.total_rows
            .saturating_sub(usize::from(self.height))
            .min(usize::from(u16::MAX)) as u16
    }

    fn scroll_down(&mut self, rows: usize) {
        self.offset = usize::from(self.offset)
            .saturating_add(rows)
            .min(usize::from(self.max_offset())) as u16;
    }

    fn scroll_up(&mut self, rows: usize) {
        self.offset = usize::from(self.offset).saturating_sub(rows) as u16;
    }
}

impl App {
    pub(super) fn apply_help(&mut self, action: VimAction) {
        match action {
            VimAction::Cancel | VimAction::Help => self.help = false,
            VimAction::Quit => self.running = false,
            VimAction::Move(Motion::Down | Motion::Right, count) => {
                self.help_view.scroll_down(count)
            }
            VimAction::Move(Motion::Up | Motion::Left, count) => self.help_view.scroll_up(count),
            VimAction::PageDown => self
                .help_view
                .scroll_down(usize::from(self.help_view.height.saturating_sub(1).max(1))),
            VimAction::PageUp => self
                .help_view
                .scroll_up(usize::from(self.help_view.height.saturating_sub(1).max(1))),
            VimAction::FileStart => self.help_view.offset = 0,
            VimAction::FileEnd => self.help_view.offset = self.help_view.max_offset(),
            _ => {}
        }
    }
}
