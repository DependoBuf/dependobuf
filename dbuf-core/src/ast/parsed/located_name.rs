//! `LocatedName` for parsed AST.

use std::ops::{Add, Deref};

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
    Str: Deref<Target = [u8]>,
    Pos: Copy + Add<usize, Output = Pos>,
{
    /// Ending position of a name.
    pub fn end(&self) -> Pos {
        self.start + self.content.len()
    }
}
