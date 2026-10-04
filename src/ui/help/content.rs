use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::super::theme::{ACCENT, TEXT};

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
                "Space / x / Enter",
                "Complete or reopen the task and its descendants",
            ),
            ("dd / 3dd", "Delete one / three task trees from selection"),
            ("u", "Restore the last deletion in this session"),
            ("t / Ctrl-p", "Cycle priority: low > mid > high > low"),
            ("ph / pm / pl", "Set high / mid / low priority"),
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
            ("Settings j / k", "Select database_path or task_view"),
            (
                "Settings Enter",
                "Edit path or cycle task_view; layouts apply live",
            ),
            ("Settings h / l", "Switch normal / split task_view"),
            ("?", "Open help; press again to close"),
            ("q / Ctrl-c", "Quit from the tree / quit anywhere"),
        ],
    },
    Section {
        title: "FIELDS / Modes & saving",
        shortcuts: &[
            (
                "Enter",
                "Save the task or setting; apply search in any mode",
            ),
            ("Esc", "Return to Normal; press again to cancel the field"),
            (
                "i / a / I / A",
                "Insert at cursor / after / first text / end",
            ),
            ("v / V", "Visual selection / select the whole field"),
            ("R", "Replace mode"),
            ("Ctrl-p", "Cycle priority in the task field"),
            ("Paste", "Insert text or replace the Visual selection"),
        ],
    },
    Section {
        title: "FIELDS / Cursor movement",
        shortcuts: &[
            ("h / l / arrows", "Move left / right"),
            (
                "0 / ^ / $",
                "Start / first nonblank / end (Home / End also work)",
            ),
            ("w / b / e", "Next word / previous word / word end"),
            ("W / B / E", "Move by space-separated WORDs"),
            ("ge / gE", "Previous word / WORD end"),
            ("gg / G", "Field start / end"),
            ("3| / %", "Display column / matching bracket"),
            ("f / F + char", "Find a character forward / backward"),
            ("t / T + char", "Stop before / after a character"),
            ("; / ,", "Repeat / reverse the last find"),
            ("Ctrl-Left/Right", "Move by word"),
        ],
    },
    Section {
        title: "FIELDS / Editing",
        shortcuts: &[
            (
                "d / c / y",
                "Delete / change / yank with a motion or object",
            ),
            ("dd / cc / yy", "Delete / change / yank the whole field"),
            ("2d3w", "Multiply counts: delete six words"),
            ("x / X", "Delete at / before the cursor"),
            ("D / C", "Delete / change to the end"),
            ("s / S", "Change characters / the whole field"),
            ("r2 / 3r2", "Replace one / three graphemes with 2"),
            ("p / P", "Put yanked text after / before the cursor"),
            ("u / Ctrl-r / .", "Undo / redo / repeat a field change"),
            (
                "gu / gU / g~",
                "Lowercase / uppercase / swap case with a motion",
            ),
            (
                "Backspace / Delete",
                "Delete before / at the cursor while typing",
            ),
            ("Ctrl-w / Ctrl-u", "Delete a word / clear while typing"),
        ],
    },
    Section {
        title: "FIELDS / Objects & selection",
        shortcuts: &[
            ("iw / aw", "Inner / around word; iW / aW for WORDs"),
            ("i / a + delimiter", "Inside / around () [] {} <> or quotes"),
            ("di{ / da}", "Delete inside / including braces"),
            ("d2i{", "Delete inside an enclosing pair with a count"),
            ("Quotes", "Single quote, double quote, or backtick"),
            ("viw", "Select a word in Visual mode"),
            ("o", "Swap the ends of the Visual selection"),
            (
                "d / c / y / r",
                "Delete / change / yank / replace a selection",
            ),
            ("u / U / ~", "Lowercase / uppercase / swap case in Visual"),
        ],
    },
];

pub(super) fn lines(width: u16) -> Vec<Line<'static>> {
    let width = usize::from(width);
    if width == 0 {
        return Vec::new();
    }
    let key_style = Style::default().fg(ACCENT).add_modifier(Modifier::BOLD);
    let heading_style = Style::default()
        .fg(TEXT)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
    let key_column = 20;
    let aligned = width >= 52;
    let mut lines = Vec::new();
    for section in SECTIONS {
        if !lines.is_empty() {
            lines.push(Line::default());
        }
        lines.extend(
            wrap(section.title, width)
                .into_iter()
                .map(|text| Line::styled(text, heading_style)),
        );
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
    lines
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
