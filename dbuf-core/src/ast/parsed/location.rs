//! Locations for parsed AST.

use std::ops::Add;

/// Position in a document.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default)]
pub struct Position {
    /// Zero-based line position in a document.
    pub line: usize,
    /// Zero-based column position in a line.
    pub column: usize,
}

/// Offset in a file.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default)]
pub struct Offset {
    /// Number of offset newline symbols.
    pub lines: usize,
    /// Number of offset symbols after last newline.
    pub columns: usize,
}

impl Add<Offset> for Position {
    type Output = Self;

    fn add(self, rhs: Offset) -> Self::Output {
        Self {
            line: self.line + rhs.lines,
            column: self.column + rhs.columns,
        }
    }
}

/// Location of a text entity.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default)]
pub struct Location<Pos> {
    /// Starting position of an entity.
    pub start: Pos,
    /// Length of an entity.
    pub length: Offset,
}

impl<Pos> Location<Pos>
where
    Pos: Add<Offset, Output = Pos>,
{
    /// Ending position of an entity.
    pub fn end(self) -> Pos {
        self.start + self.length
    }
}
