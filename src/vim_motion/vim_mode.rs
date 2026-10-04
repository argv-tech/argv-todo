#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VimMode {
    #[default]
    Normal,
    Insert,
    Visual,
    VisualLine,
    Replace,
}

impl VimMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
            Self::VisualLine => "V-LINE",
            Self::Replace => "REPLACE",
        }
    }

    pub(crate) fn typing(self) -> bool {
        matches!(self, Self::Insert | Self::Replace)
    }

    pub(crate) fn visual(self) -> bool {
        matches!(self, Self::Visual | Self::VisualLine)
    }
}
