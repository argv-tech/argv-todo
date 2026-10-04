mod details;
mod form;
mod paths;
mod render;

pub(super) use render::draw_config;

#[cfg(test)]
pub(super) use render::CONFIG_LOGO;

#[cfg(test)]
#[path = "../../../tests/unit/ui/config.rs"]
mod tests;
