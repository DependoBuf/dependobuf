//! Module exports:
//! * `LocationHelpers`, helpers for Location type.
//!

use tower_lsp::lsp_types::Range;
use tower_lsp::lsp_types::{self};

use dbuf_core::ast::parsed::location::{self, Offset};

pub type Position = location::Offset;
pub type Location = location::Location<Position>;

/// Helpers for `dbuf-core::Position` type.
pub trait PositionHelper {
    /// Creates new position.
    fn new(lines: usize, columns: usize) -> Self;
    /// Get line of position.
    fn get_line(&self) -> usize;
    /// Get column of position.
    fn get_column(&self) -> usize;
    /// Get column as mutable.
    fn get_column_mut(&mut self) -> &mut usize;
    /// Convers Position to `lsp_types::Position`;
    fn to_lsp(&self) -> lsp_types::Position;
}

impl PositionHelper for Position {
    fn new(lines: usize, columns: usize) -> Self {
        Position { lines, columns }
    }
    fn get_line(&self) -> usize {
        self.lines
    }
    fn get_column(&self) -> usize {
        self.columns
    }
    fn get_column_mut(&mut self) -> &mut usize {
        &mut self.columns
    }
    fn to_lsp(&self) -> lsp_types::Position {
        lsp_types::Position {
            line: u32::try_from(self.lines).unwrap(),
            character: u32::try_from(self.columns).unwrap(),
        }
    }
}

fn to_position(p: lsp_types::Position) -> Position {
    Position {
        columns: p.character as usize,
        lines: p.line as usize,
    }
}

/// Helpers for `dbuf-core::Location` type.
pub trait LocationHelper {
    /// Type for start of position.
    type Pos: PositionHelper;

    /// Returns empty location. Typically ((0, 0), (0, 0))
    fn new_empty() -> Self;
    /// Returns location with start and end.
    fn new(start: Self::Pos, end: Self::Pos) -> Self;
    /// Convers Location to `lsp_types::Range`;
    fn to_lsp(&self) -> Range;
    /// Check if cursor position in location.
    ///
    /// If `p == self.end`, returns true, corresponding
    /// to `lsp_type::Range` specification.
    fn contains(&self, p: lsp_types::Position) -> bool;
    /// Checks if location intersects with range.
    fn intersects(&self, r: Range) -> bool;
    /// Get start of location.
    fn get_start(&self) -> Self::Pos;
    /// Get end of location.
    fn get_end(&self) -> Self::Pos;
    /// Sets start of location and resets end.
    fn reset_start(&mut self, start: Self::Pos);
    /// Sets end of location.
    fn set_end(&mut self, end: Self::Pos);
}

impl LocationHelper for Location {
    type Pos = Position;

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

    fn new(start: Self::Pos, end: Self::Pos) -> Self {
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

    fn get_start(&self) -> Self::Pos {
        self.start
    }

    fn get_end(&self) -> Self::Pos {
        self.end()
    }

    fn reset_start(&mut self, start: Self::Pos) {
        self.start = start;
        self.length = Offset::default();
    }

    fn set_end(&mut self, end: Self::Pos) {
        if end.lines == self.start.lines {
            self.length.lines = 0;
            self.length.columns = end.columns - self.start.columns;
        } else {
            self.length.lines = end.lines - self.start.lines;
            self.length.columns = end.columns;
        }
    }
}
