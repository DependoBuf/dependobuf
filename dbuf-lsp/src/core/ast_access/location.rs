//! Module exports:
//! * LocationHelpers, helpers for Location type.
//!

use tower_lsp::lsp_types::Range;
use tower_lsp::lsp_types::{self};

use dbuf_core::ast::parsed::location::{self, Offset};

pub type Position = location::Offset;
pub type Location = location::Location<Position>;

/// Helpers for dbuf-core::Position type.
pub trait PositionHelpers {
    /// Creates new position.
    fn new(lines: u32, columns: u32) -> Self;
    /// Get line of position.
    fn get_line(&self) -> u32;
    /// Get column of position.
    fn get_column(&self) -> u32;
    /// Get column as mutable.
    fn get_column_mut(&mut self) -> &mut usize;
    /// Convers Position to lsp_types::Position;
    fn to_lsp(&self) -> lsp_types::Position;
}

impl PositionHelpers for Position {
    fn new(lines: u32, columns: u32) -> Self {
        Position {
            lines: lines as usize,
            columns: columns as usize,
        }
    }
    fn get_line(&self) -> u32 {
        self.lines as u32
    }
    fn get_column(&self) -> u32 {
        self.columns as u32
    }
    fn get_column_mut(&mut self) -> &mut usize {
        &mut self.columns
    }
    fn to_lsp(&self) -> lsp_types::Position {
        lsp_types::Position {
            line: self.lines as u32,
            character: self.columns as u32,
        }
    }
}

fn to_position(p: lsp_types::Position) -> Position {
    Position {
        columns: p.character as usize,
        lines: p.line as usize,
    }
}

/// Helpers for dbuf-core::Location type.
pub trait LocationHelpers {
    /// Returns empty location. Typically ((0, 0), (0, 0))
    fn new_empty() -> Self;
    /// Returns location with start and end.
    fn new(start: Position, end: Position) -> Self;
    /// Convers Location to lsp_types::Range;
    fn to_lsp(&self) -> Range;
    /// Check if cursor position in location.
    ///
    /// If `p == self.end`, returns true, corresponding
    /// to lsp_type::Range specification.
    fn contains(&self, p: lsp_types::Position) -> bool;
    /// Checks if location intersects with range.
    fn intersects(&self, r: Range) -> bool;
    /// Get start of location.
    fn get_start(&self) -> Position;
    /// Get end of location.
    fn get_end(&self) -> Position;
    /// Sets start of location and resets end.
    fn reset_start(&mut self, start: Position);
    /// Sets end of location.
    fn set_end(&mut self, end: Position);
}

impl LocationHelpers for Location {
    fn new_empty() -> Self {
        Self {
            start: Position {
                lines: 0,
                columns: 0,
            },
            length: Position {
                lines: 0,
                columns: 0,
            },
        }
    }

    fn new(start: Position, end: Position) -> Self {
        let mut ans = Self::new_empty();
        ans.reset_start(start);
        ans.set_end(end);
        ans
    }

    fn to_lsp(&self) -> Range {
        Range {
            start: self.start.to_lsp(),
            end: self.end().to_lsp(),
        }
    }

    fn contains(&self, p: lsp_types::Position) -> bool {
        let target = to_position(p);
        self.start <= target && target <= self.end()
    }

    fn intersects(&self, r: Range) -> bool {
        if self.contains(r.start) || self.contains(r.end) {
            true
        } else {
            let start = to_position(r.start);
            let end = to_position(r.end);
            start <= self.start && self.end() <= end
        }
    }

    fn get_start(&self) -> Position {
        self.start
    }

    fn get_end(&self) -> Position {
        self.end()
    }

    fn reset_start(&mut self, start: Position) {
        self.start = start;
        self.length = Offset::default();
    }

    fn set_end(&mut self, end: Position) {
        if end.lines == self.start.lines {
            self.length.lines = 0;
            self.length.columns = end.columns - self.start.columns;
        } else {
            self.length.lines = end.lines - self.start.lines;
            self.length.columns = end.columns
        }
    }
}
