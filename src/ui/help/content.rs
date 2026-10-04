use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::super::theme::ACCENT;

struct Section {
    title: &'static str,
    shortcuts: &'static [(&'static str, &'static str)],
}

const SECTIONS: &[Section] = &[
    Section {
        title: "TASKS / Navigation",
        shortcuts: &[
            ("j / k", "Next / previous task (or Up / Down)"),
            ("l / h", "First child / parent; stay if there is no child"),
            ("Enter", "Collapse / expand the selected task's children"),
            ("gg / G", "First / last task"),
            ("3j / 2k", "Repeat a movement"),
            (
                "Tab / Shift-Tab",
                "Switch panes in Split; skip ghost parents",
            ),
        ],
    },
    Section {
        title: "TASKS / Create & change",
        shortcuts: &[
            ("i / a", "Add a child; add a root when the list is empty"),
            ("o / O", "Add a sibling below / above"),
            ("e / cc", "Edit the selected task"),
            (
                "Space / x",
                "Complete or reopen the task and its descendants",
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
        title: "APP / Search & settings",
        shortcuts: &[
            ("/", "Search all titles; keep ancestors visible"),
            (
                "Esc",
                "Clear an applied search, otherwise open configuration",
            ),
            ("Ctrl-r", "Reload tasks from disk"),
            (
                "Settings j / k",
                "Select a setting; Tab / Shift-Tab cycles settings",
            ),
            (
                "Settings Enter",
                "Edit path or change a setting; changes apply live",
            ),
            (
                "Settings h / l",
                "Cycle the selected setting backward / forward",
            ),
            ("?", "Open help; press again to close"),
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

// Wrap before rendering so scrolling and page limits count actual terminal rows.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0;
    for word in text.split_whitespace() {
        if !line.is_empty() && used + 1 + word.width() > width {
            lines.push(std::mem::take(&mut line));
            used = 0;
        }
        if !line.is_empty() {
            line.push(' ');
            used += 1;
        }
        for grapheme in word.graphemes(true) {
            if used + grapheme.width() > width && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
                used = 0;
            }
            line.push_str(grapheme);
            used += grapheme.width();
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
#[path = "../../../tests/unit/ui/help_content.rs"]
mod tests;
