//! Locations for parsed AST.

use std::ops::Add;

/// Offset in a file.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default)]
pub struct Offset {
    /// Number of offset newline symbols.
    pub lines: usize,
    /// Number of offset symbols after last newline.
    pub columns: usize,
}

impl Add<Offset> for Offset {
    type Output = Self;

    fn add(self, rhs: Offset) -> Self::Output {
        if rhs.lines == 0 {
            Self {
                lines: self.lines,
                columns: self.columns + rhs.columns,
            }
        } else {
            Self {
                lines: self.lines + rhs.lines,
                columns: rhs.columns,
            }
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
