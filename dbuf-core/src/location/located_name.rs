//! Module exports `LocatedName` a single line name with location.
//!

use std::{fmt, ops::Add};

use crate::location::{Location, Offset};

/// Single line name with location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedName<Str, Pos> {
    /// Name content.
    pub content: Str,
    /// Starting position of a name.
    pub start: Pos,
}

impl<Str, Pos> LocatedName<Str, Pos>
where
    Str: AsRef<str>,
    Pos: Copy + Add<usize, Output = Pos>,
{
    /// Ending position of a name.
    ///
    /// Assumes name contains no newline characters.
    pub fn end(&self) -> Pos {
        self.start + self.content.as_ref().len()
    }
}

impl<Str, Pos> AsRef<str> for LocatedName<Str, Pos>
where
    Str: AsRef<str>,
{
    fn as_ref(&self) -> &str {
        self.content.as_ref()
    }
}

impl<Str, Pos> fmt::Display for LocatedName<Str, Pos>
where
    Str: AsRef<str>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.content.as_ref().fmt(f)
    }
}

impl<Str: AsRef<str>, Pos: Clone> From<&LocatedName<Str, Pos>> for Location<Pos> {
    fn from(value: &LocatedName<Str, Pos>) -> Self {
        Location {
            start: value.start.clone(),
            length: Offset {
                lines: 0,
                columns: value.content.as_ref().len(),
            },
        }
    }
}
