//! `LocatedName` for parsed AST.

use std::{fmt, ops::Add};

/// Single line name with location.
#[derive(Clone, Debug)]
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
