use super::App;
use crate::vim_motion::{Motion, VimAction};

#[derive(Default)]
pub(crate) struct HelpView {
    offset: u16,
    height: u16,
    total_rows: usize,
    section: usize,
    section_count: Option<usize>,
}

impl HelpView {
    pub(crate) fn offset(&self) -> u16 {
        self.offset
    }

    pub(crate) fn total_rows(&self) -> usize {
        self.total_rows
    }

    pub(crate) fn selected_section(&self) -> usize {
        self.section
    }

    pub(crate) fn has_guide(&self) -> bool {
        self.section_count.is_some()
    }

    pub(crate) fn set_sections(&mut self, count: Option<usize>) {
        self.section_count = count.filter(|&count| count > 0);
        if let Some(count) = self.section_count {
            self.section = self.section.min(count - 1);
        }
    }

    fn select_section(&mut self, section: usize) {
        if let Some(count) = self.section_count {
            self.section = section.min(count - 1);
            self.offset = 0;
        }
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
            VimAction::SwitchPane | VimAction::SwitchPaneBackward => {
                if let Some(count) = self.help_view.section_count {
                    let selected = self.help_view.section;
                    let next = if action == VimAction::SwitchPane {
                        (selected + 1) % count
                    } else {
                        (selected + count - 1) % count
                    };
                    self.help_view.select_section(next);
                }
            }
            VimAction::Move(Motion::Down | Motion::Right, count) if self.help_view.has_guide() => {
                self.help_view
                    .select_section(self.help_view.section.saturating_add(count));
            }
            VimAction::Move(Motion::Up | Motion::Left, count) if self.help_view.has_guide() => {
                self.help_view
                    .select_section(self.help_view.section.saturating_sub(count));
            }
            VimAction::Move(Motion::Down | Motion::Right, count) => {
                self.help_view.scroll_down(count)
            }
            VimAction::Move(Motion::Up | Motion::Left, count) => self.help_view.scroll_up(count),
            VimAction::PageDown => {
                self.help_view
                    .scroll_down(usize::from(self.help_view.height.saturating_sub(1).max(1)));
            }
            VimAction::PageUp => {
                self.help_view
                    .scroll_up(usize::from(self.help_view.height.saturating_sub(1).max(1)));
            }
            VimAction::FileStart if self.help_view.has_guide() => self.help_view.select_section(0),
            VimAction::FileEnd if self.help_view.has_guide() => {
                self.help_view.select_section(usize::MAX)
            }
            VimAction::FileStart => self.help_view.offset = 0,
            VimAction::FileEnd => self.help_view.offset = self.help_view.max_offset(),
            _ => {}
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/app/help.rs"]
mod tests;
