use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use unicode_width::UnicodeWidthStr;

use super::super::{text::wrap, theme::ACCENT};

struct Section {
    title: &'static str,
    shortcuts: &'static [(&'static str, &'static str)],
}

const SECTIONS: &[Section] = &[
    Section {
        title: "TASKS / Navigation",
        shortcuts: &[
            ("j / k", "Next / previous task (or Up / Down)"),
            (
                "l / h",
                "Child / parent; in Projects, switch tabs at tree edges",
            ),
            (
                "Right / Left",
                "First child / parent; stay if there is no child",
            ),
            ("Enter", "Collapse / expand the selected task's children"),
            ("gg / G", "First / last task"),
            ("Home / End", "First / last task"),
            ("3j / 2k", "Repeat a movement"),
            ("Tab / Shift-Tab", "Switch project tabs or Split panes"),
        ],
    },
    Section {
        title: "TASKS / Create & change",
        shortcuts: &[
            ("i / a", "Add a child; add a root when the list is empty"),
            ("o / O", "Add a sibling below / above"),
            ("P", "Create a project and open its tab"),
            (
                "C",
                "Convert a top-level task to a project; keep its whole tree",
            ),
            (
                "m",
                "Move a task tree to a project; Enter confirms, Esc cancels",
            ),
            ("Picker j / k", "Select a destination (or Up / Down)"),
            ("Picker Tab/S-Tab", "Next / previous destination"),
            ("Picker Home/End", "First / last destination"),
            ("e / cc", "Edit the selected task"),
            (
                "n",
                "Edit the selected task's multi-line note in the bottom section",
            ),
            ("Note Enter", "Add a line while typing; save in Normal mode"),
            ("Note Ctrl-s", "Save the note from any mode"),
            ("Note Up / Down", "Move between note lines while typing"),
            ("Note j / k", "Move between note lines in Normal mode"),
            (
                "Note 0 / $",
                "Start / end of the current line in Normal mode",
            ),
            ("Note gg / G", "Start / end of the note in Normal mode"),
            (
                "Note Esc",
                "Return to Normal; press again to discard the draft",
            ),
            (
                "Space / x",
                "Complete or reopen the task and its descendants",
            ),
            (
                "Alt-Enter",
                "Clear completed tasks; current project in Projects; Settings chooses soft or hard deletion",
            ),
            ("dd / 3dd", "Delete one / three task trees from selection"),
            ("u", "Restore the last deletion in this session"),
            ("t / Ctrl-p", "Cycle priority: low > mid > high > low"),
            ("ph / pm / pl", "Set high / mid / low priority"),
            ("Shift-J / K", "Move a task tree down / up in manual sort"),
            (
                "Shift-H / L",
                "Outdent / indent under previous sibling in manual sort",
            ),
        ],
    },
    Section {
        title: "ARCHIVE / Restore & clear",
        shortcuts: &[
            (
                "A",
                "Open soft-deleted tasks from all projects, with saved notes",
            ),
            ("j / k", "Select an archived task (or Up / Down)"),
            ("gg / G", "First / last archived task (or Home / End)"),
            ("Ctrl-r", "Reload archived tasks"),
            (
                "r",
                "Restore selected task, archived descendants and archived parents; keep completion and notes",
            ),
            (
                "D (Shift-d)",
                "Permanently clear all archived tasks from all projects; cannot undo",
            ),
            ("Enter", "Confirm permanent archive cleanup"),
            ("Esc", "Cancel cleanup; otherwise return to active tasks"),
            (
                "A",
                "Return to active tasks; cancel a pending cleanup first",
            ),
        ],
    },
    Section {
        title: "SETTINGS / Preferences",
        shortcuts: &[
            (
                "j / k",
                "Select across Storage, Display and Task behavior (or Up / Down)",
            ),
            (
                "Tab / Shift-Tab",
                "Next / previous setting; skip category headings",
            ),
            ("gg / G", "First / last setting (or Home / End)"),
            (
                "Enter / e / i / a",
                "Edit database path or cycle the selected preference",
            ),
            (
                "h / l",
                "Cycle preference backward / forward (or Left / Right)",
            ),
            (
                "Enter (path)",
                "Save database path for next launch; other preferences apply immediately",
            ),
            (
                "Esc (path)",
                "Return to Normal; press again to discard the draft",
            ),
            ("Esc", "Return to tasks"),
        ],
    },
    Section {
        title: "APP / Search & help",
        shortcuts: &[
            (
                "/",
                "Search titles in the current view; keep ancestors visible",
            ),
            (
                "Esc",
                "Clear an applied search, otherwise open configuration",
            ),
            ("Ctrl-r", "Reload tasks from disk"),
            ("?", "Open help; press again to close"),
            (
                "Help j / k",
                "Select a section; scroll on narrow screens (or arrows)",
            ),
            (
                "Help Tab/S-Tab",
                "Next / previous section when the guide is shown",
            ),
            (
                "Help gg / G",
                "First / last section or scroll end (or Home / End)",
            ),
            ("PgUp / PgDn", "Scroll help by a page"),
            ("Help Esc / ?", "Close help"),
            ("q / Ctrl-c", "Quit from the tree / quit anywhere"),
        ],
    },
];

pub(super) struct SectionPosition {
    pub(super) title: &'static str,
    pub(super) row: usize,
}

#[derive(Default)]
pub(super) struct Document {
    pub(super) lines: Vec<Line<'static>>,
    pub(super) sections: Vec<SectionPosition>,
}

impl Document {
    pub(super) fn section_range(&self, index: usize) -> std::ops::Range<usize> {
        let start = self.sections.get(index).map_or(0, |section| section.row);
        let end = self
            .sections
            .get(index + 1)
            .map_or(self.lines.len(), |section| section.row.saturating_sub(1));
        start..end
    }
}

pub(super) fn document(width: u16) -> Document {
    let width = usize::from(width);
    if width == 0 {
        return Document::default();
    }
    let key_style = Style::default().fg(ACCENT).add_modifier(Modifier::BOLD);
    let heading_style = Style::default()
        .fg(ACCENT)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
    let key_column = 20;
    let aligned = width >= 52;
    let mut lines = Vec::new();
    let mut sections = Vec::new();
    for section in SECTIONS {
        if !lines.is_empty() {
            lines.push(Line::default());
        }
        sections.push(SectionPosition {
            title: section.title,
            row: lines.len(),
        });
        lines.extend(
            wrap(&section.title.to_uppercase(), width)
                .into_iter()
                .map(|text| Line::styled(text, heading_style)),
        );
        lines.push(Line::default());
        for &(keys, description) in section.shortcuts {
            if aligned {
                for (index, text) in wrap(description, width - key_column)
                    .into_iter()
                    .enumerate()
                {
                    let key = if index == 0 { keys } else { "" };
                    lines.push(Line::from(vec![
                        Span::styled(key, key_style),
                        Span::raw(" ".repeat(key_column - key.width())),
                        Span::raw(text),
                    ]));
                }
            } else {
                lines.extend(
                    wrap(keys, width)
                        .into_iter()
                        .map(|text| Line::styled(text, key_style)),
                );
                let indent = 2.min(width.saturating_sub(1));
                lines.extend(
                    wrap(description, width - indent)
                        .into_iter()
                        .map(|text| Line::from(format!("{}{text}", " ".repeat(indent)))),
                );
            }
        }
    }
    Document { lines, sections }
}

#[cfg(test)]
#[path = "../../../tests/unit/ui/help_content.rs"]
mod tests;
