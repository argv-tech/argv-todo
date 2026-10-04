mod details;
mod form;
mod paths;
mod render;

pub(super) use render::draw_config;

#[cfg(test)]
pub(super) use super::branding::LOGO as CONFIG_LOGO;

#[cfg(test)]
#[path = "../../../tests/unit/ui/config.rs"]
mod tests;
