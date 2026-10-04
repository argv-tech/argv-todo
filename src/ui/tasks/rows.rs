use crate::{
    app::{App, TaskPane},
    config::SortOrder,
    vim_motion::InputTarget,
};

pub(super) struct DisplayRow {
    pub(super) index: Option<usize>,
    pub(super) depth: usize,
    pub(super) ghost: bool,
}

pub(super) fn display_rows(app: &App, pane: TaskPane) -> (Vec<DisplayRow>, Option<usize>) {
    let mut visible: Vec<_> = app
        .rows_for_pane(pane)
        .into_iter()
        .map(|row| DisplayRow {
            index: Some(row.index),
            depth: row.depth,
            ghost: row.ghost,
        })
        .collect();
    let adding = pane == app.task_pane()
        && app.vim.input() == Some(InputTarget::Task)
        && app.editing_id.is_none();
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
                    && match app.sort_order() {
                        SortOrder::Priority => {
                            todo.priority > app.input_priority
                                || (todo.priority == app.input_priority
                                    && relative_position
                                        .is_some_and(|position| todo.position >= position))
                        }
                        SortOrder::Manual => {
                            relative_position.is_some_and(|position| todo.position >= position)
                        }
                    }
            })
            .unwrap_or(end);
        visible.insert(
            position,
            DisplayRow {
                index: None,
                depth,
                ghost: false,
            },
        );
        Some(position)
    } else {
        None
    };
    (visible, draft)
}
