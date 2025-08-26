//! Locations for parsed AST.

use std::ops::{Add, Range, Sub};

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

impl Add<usize> for Offset {
    type Output = Offset;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            lines: self.lines,
            columns: self.columns + rhs,
        }
    }
}

impl Sub<Offset> for Offset {
    type Output = Option<Self>;

    // For calculating length
    fn sub(self, rhs: Offset) -> Self::Output {
        if self < rhs {
            None
        } else if self.lines == rhs.lines {
            Some(Self {
                lines: 0,
                columns: self.columns - rhs.columns,
            })
        } else {
            Some(Self {
                lines: self.lines - rhs.lines,
                columns: self.columns,
            })
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

#[derive(Debug, Clone, Copy)]
pub struct IncorrectRange;

impl<Pos> TryFrom<Range<Pos>> for Location<Pos>
where
    Pos: Sub<Pos, Output = Option<Offset>> + Copy,
{
    type Error = IncorrectRange;

    fn try_from(range: Range<Pos>) -> Result<Self, Self::Error> {
        Ok(Location {
            start: range.start,
            length: (range.end - range.start).ok_or(Self::Error {})?,
        })
    }
}
