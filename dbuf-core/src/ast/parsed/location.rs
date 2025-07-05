//! Locations for parsed AST.

use std::ops::{Add, Range, Sub};

use chumsky::span::Span;

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

impl Sub<Offset> for Offset {
    type Output = Self;

    // For calculating length
    fn sub(self, rhs: Offset) -> Self::Output {
        if self.lines == rhs.lines {
            Self {
                lines: 0,
                columns: self.columns - rhs.columns,
            }
        } else {
            Self {
                lines: self.lines - rhs.lines,
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

impl<Pos: Copy> Span for Location<Pos>
where
    Pos: Add<Offset, Output = Pos>,
    Pos: Sub<Pos, Output = Offset>,
{
    type Context = ();
    type Offset = Pos;

    fn new(_context: Self::Context, range: Range<Self::Offset>) -> Self {
        Location {
            start: range.start,
            length: range.end - range.start,
        }
    }

    fn context(&self) -> Self::Context {
        ()
    }

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.start + self.length
    }

    fn to_end(&self) -> Self
    where
        Self: Sized,
    {
        Self::new(self.context(), self.end()..self.end())
    }

    fn union(&self, other: Self) -> Self
    where
        Self::Context: PartialEq + core::fmt::Debug,
        Self::Offset: Ord,
        Self: Sized,
    {
        std::assert_eq!(
            self.context(),
            other.context(),
            "tried to union two spans with different contexts"
        );
        Self::new(
            self.context(),
            self.start().min(other.start())..self.end().max(other.end()),
        )
    }
}
