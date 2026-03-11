//! Module exports:
//!   * `Location` - specification for `crate::location::Location` where
//!     starting position is offset.
//!   * `Located` trait for lexers that can return `Location` of tokens.
//!
//! Module implements:
//!   * `chumsky::Span` trait for `Location`

// uses only `Offset` from there
use chumsky::span::Span;

use crate::location::Location;
use crate::location::Offset;

/// Trait for lexers that can return `Location` of just taken token.
pub trait Locatable {
    /// Returns `Location` of last Token.
    ///
    /// The behavior is undefined if no token taken.
    fn location(&self) -> Location<Offset>;
}

impl Location<Offset> {
    #[must_use]
    pub(super) fn new(start: Offset, end: Offset) -> Option<Location<Offset>> {
        (end - start).map(|length| Location { start, length })
    }

    #[must_use]
    pub(super) fn point(point: Offset) -> Location<Offset> {
        Location {
            start: point,
            length: Offset {
                lines: 0,
                columns: 0,
            },
        }
    }
}

impl Span for Location<Offset> {
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
        Self::end(self)
    }
}
