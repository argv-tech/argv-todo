use crate::{app::App, vim_motion::VimMode};

pub(super) struct DisplayRow {
    pub(super) index: Option<usize>,
    pub(super) depth: usize,
}

pub(super) fn display_rows(app: &App) -> (Vec<DisplayRow>, Option<usize>) {
    let mut visible: Vec<_> = app
        .visible_rows()
        .into_iter()
        .map(|row| DisplayRow {
            index: Some(row.index),
            depth: row.depth,
        })
        .collect();
    let adding = app.vim.mode() == VimMode::Insert && app.editing_id.is_none();
    let draft = if adding {
        let (start, end, depth) = app
            .adding_parent
            .and_then(|id| {
                visible.iter().enumerate().find_map(|(row, task)| {
                    (task.index.is_some_and(|index| app.todos[index].id == id))
                        .then_some((row, task.depth))
                })
            })
            .map(|(parent, depth)| {
                let position = visible
                    .iter()
                    .enumerate()
                    .skip(parent + 1)
                    .find(|(_, task)| task.depth <= depth)
                    .map_or(visible.len(), |(row, _)| row);
                (parent + 1, position, depth + 1)
            })
            .unwrap_or((0, visible.len(), 0));
        let relative_position = app.adding_relative.and_then(|(id, above)| {
            app.todos
                .iter()
                .find(|todo| todo.id == id)
                .map(|todo| todo.position + i64::from(!above))
        });
        let position = (start..end)
            .find(|&row| {
                let Some(index) = visible[row].index else {
                    return false;
                };
                let todo = &app.todos[index];
                visible[row].depth == depth
                    && (todo.priority > app.input_priority
                        || (todo.priority == app.input_priority
                            && relative_position.is_some_and(|position| todo.position >= position)))
            })
            .unwrap_or(end);
        visible.insert(position, DisplayRow { index: None, depth });
        Some(position)
    } else {
        None
    };
    (visible, draft)
}
