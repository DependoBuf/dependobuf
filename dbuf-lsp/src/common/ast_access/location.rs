pub use dbuf_core::location::Location;
pub use dbuf_core::location::Position;

use tower_lsp::lsp_types;
use tower_lsp::lsp_types::Range;

pub trait PositionHelpers {
    fn to_lsp(&self) -> lsp_types::Position;
}

pub trait LocationHelpers {
    fn new_empty() -> Self;
    fn to_lsp(&self) -> Range;
    fn contains(&self, p: &lsp_types::Position) -> bool;
}

impl PositionHelpers for Position {
    fn to_lsp(&self) -> lsp_types::Position {
        lsp_types::Position {
            line: self.line,
            character: self.character,
        }
    }
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

    fn contains(&self, p: &lsp_types::Position) -> bool {
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
        return p.character <= self.end.character;
    }
}
