#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    #[default]
    Normal,
    Insert,
    Search,
}

impl VimMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Search => "SEARCH",
        }
    }
}
