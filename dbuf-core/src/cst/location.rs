//! Module exports:
//!   * `Location` struct that used everywhere in  `CST` module.
//!   * `Located` trait for lexers that can return `Location` of tokens.
//!

// uses only `Offset` from there
use crate::ast::parsed::location;
use chumsky::span::Span;

/// Offset type for `CST` locations.
pub type Offset = location::Offset;

/// Location for `Lexer` and `CST` that
/// supports multiline locations.
///
/// Holds invariant that (end - start) is not `None`.
#[derive(Clone, Debug)]
pub struct Location {
    start: Offset,
    end: Offset,
}

/// Trait for lexers that can return `Location` of just taken token.
pub trait Locatable {
    /// Returns `Location` of last Token.
    ///
    /// The behavior is undefined if no token taken.
    fn location(&self) -> Location;
}

impl Location {
    #[must_use]
    pub fn new(start: Offset, end: Offset) -> Option<Location> {
        if (end - start).is_some() {
            Some(Location { start, end })
        } else {
            None
        }
    }

    #[must_use]
    pub fn point(point: Offset) -> Location {
        Location {
            start: point,
            end: point,
        }
    }

    #[must_use]
    pub fn start(&self) -> Offset {
        self.start
    }

    #[must_use]
    pub fn end(&self) -> Offset {
        self.end
    }
}

impl Span for Location {
    type Context = ();

    type Offset = Offset;

    fn new(_context: Self::Context, range: std::ops::Range<Self::Offset>) -> Self {
        Self::new(range.start, range.end).expect("correct increasing range")
    }

    fn context(&self) -> Self::Context {}

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}
