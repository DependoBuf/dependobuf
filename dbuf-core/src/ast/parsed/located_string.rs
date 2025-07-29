//! `LocatedString` for parsed AST.

use std::ops::{Add, Deref};

/// Single line string with location.
pub struct LocatedString<String, Pos> {
    /// String content.
    pub content: String,
    /// Starting position of string.
    pub start: Pos,
}

impl<String, Pos> LocatedString<String, Pos>
where
    String: Deref<Target = [u8]>,
    Pos: Copy + Add<usize, Output = Pos>,
{
    /// Ending position of a string.
    pub fn end(&self) -> Pos {
        self.start + self.content.len()
    }
}
