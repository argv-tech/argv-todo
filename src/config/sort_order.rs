use anyhow::{Result, bail};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SortOrder {
    #[default]
    Priority,
    Manual,
    Deadline,
}

impl SortOrder {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Priority => "priority",
            Self::Manual => "manual",
            Self::Deadline => "deadline",
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            Self::Priority => Self::Manual,
            Self::Manual => Self::Deadline,
            Self::Deadline => Self::Priority,
        }
    }

    pub(super) fn parse(value: &str) -> Result<Self> {
        match value {
            "priority" => Ok(Self::Priority),
            "manual" => Ok(Self::Manual),
            "deadline" => Ok(Self::Deadline),
            _ => bail!("sort_order must be priority, manual, or deadline"),
        }
    }
}
