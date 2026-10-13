mod picker;
mod tabs;

pub(super) use picker::draw_picker;
pub(super) use tabs::draw_tabs;

#[cfg(test)]
#[path = "../../../tests/unit/ui/projects.rs"]
mod tests;
