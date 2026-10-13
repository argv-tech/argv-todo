use anyhow::{Result, bail};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TaskView {
    #[default]
    Normal,
    Split,
    Projects,
}

impl TaskView {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Split => "split",
            Self::Projects => "projects",
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            Self::Normal => Self::Split,
            Self::Split => Self::Projects,
            Self::Projects => Self::Normal,
        }
    }

    pub(super) fn parse(value: &str) -> Result<Self> {
        match value {
            // Removed layouts fall back to the normal view for existing configs.
            "normal" | "nested" => Ok(Self::Normal),
            "split" => Ok(Self::Split),
            "projects" => Ok(Self::Projects),
            _ => bail!("task_view must be normal, split, or projects"),
        }
    }
}
