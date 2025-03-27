//! Locations for parsed AST.

/// Poosition in a document.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default)]
pub struct Position {
    /// Zero-based line position in a document.
    pub line: u32,
    /// Zero-based character position in a line.
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Position {
        Position { line, character }
    }
}

/// Location in a document.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct Location {
    /// Zero-based start of location.
    pub start: Position,
    /// Zero-based end of location, which is not included in the location.
    /// For example, the location `((0, 0), (0, 2))` includes only two characters at positions `(0, 0)` and `(0, 1)`
    ///
    /// If a location has more than one line, the end position must be on the same `line` as the last character in the location.
    pub end: Position,
}

impl Location {
    pub fn new(start: Position, end: Position) -> Location {
        Location { start, end }
    }
}
