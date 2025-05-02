//! Module exports:
//! * LocationHelpers, helpers for Location type.
//!

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

use tower_lsp::lsp_types;
use tower_lsp::lsp_types::Range;

/// Helpers for dbuf-core::Position type.
trait PositionHelpers {
    /// Convers Position to lsp_types::Position;
    fn to_lsp(&self) -> lsp_types::Position;
}

impl PositionHelpers for Position {
    fn to_lsp(&self) -> lsp_types::Position {
        lsp_types::Position {
            line: self.line,
            character: self.character,
        }
    }
}

/// Helpers for dbuf-core::Location type.
pub trait LocationHelpers {
    /// Returns empty location. Typically ((0, 0), (0, 0))
    fn new_empty() -> Self;
    /// Convers Location to lsp_types::Range;
    fn to_lsp(&self) -> Range;
    /// Check if cursor position in location.
    ///
    /// If `p == self.end`, returns true, corresponding
    /// to lsp_type::Range specification.
    fn contains(&self, p: lsp_types::Position) -> bool;
}

impl LocationHelpers for Location {
    fn new_empty() -> Location {
        Location::new(Position::new(0, 0), Position::new(0, 0))
    }

    fn to_lsp(&self) -> Range {
        Range {
            start: self.start.to_lsp(),
            end: self.end.to_lsp(),
        }
    }

    fn contains(&self, p: lsp_types::Position) -> bool {
        if self.start.line > p.line {
            return false;
        }
        if self.start.line == p.line && self.start.character > p.character {
            return false;
        }
        if self.end.line < p.line {
            return false;
        }
        if self.end.line > p.line {
            return true;
        }
        p.character <= self.end.character
    }
}
