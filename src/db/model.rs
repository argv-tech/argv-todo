use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Priority {
    High = 1,
    #[default]
    Mid = 3,
    Low = 5,
}

impl Priority {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Mid => "mid",
            Self::Low => "low",
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            Self::Low => Self::Mid,
            Self::Mid => Self::High,
            Self::High => Self::Low,
        }
    }
}

impl ToSql for Priority {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as i64)))
    }
}

impl FromSql for Priority {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            1 => Ok(Self::High),
            3 => Ok(Self::Mid),
            5 => Ok(Self::Low),
            value => Err(FromSqlError::OutOfRange(value)),
        }
    }
}

pub(crate) const DEFAULT_PRIORITY: Priority = Priority::Mid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Todo {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) done: bool,
    pub(crate) parent_id: Option<i64>,
    pub(crate) priority: Priority,
    pub(crate) position: i64,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}
