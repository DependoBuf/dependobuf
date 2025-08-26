//! Module exports
//! * `ConvertibleToString` trait, wich allows any type conversation to `LocatedName`.
//! * `LocNameHelper` trait with helpfull funtions for `LocatedName`.
//!

use super::location::{LocationHelper, PositionHelper};

use tower_lsp::lsp_types;

use super::{Loc, Str};

/// Trait for types, that can be converted to `LocatedName`.
/// TODO: remove such API.
pub trait ConvertibleToString {
    fn to_loc_string(&self) -> Str;
}

type Pos = <Loc as LocationHelper>::Pos;

/// Helpers for `dbuf-core::LocatedName`.
pub trait LocNameHelper {
    /// Constructs `LocatedName`.
    fn new(string: &str, location: Loc) -> Self;
    /// Returns string's len.
    fn len(&self) -> usize;
    /// Returns string's location.
    fn get_location(&self) -> Loc;
    /// Returns if positions in string's location.
    fn contains(&self, p: lsp_types::Position) -> bool {
        self.get_location().contains(p)
    }
    /// Constructs `LocatedName` with empty location.
    /// TODO: remove such API.
    fn unsafe_new(string: &str) -> Self;
    /// Sets begin of string's location.
    /// TODO: remove such API.
    fn set_location_start(&mut self, start: Pos);
}

impl LocNameHelper for Str {
    fn new(string: &str, location: Loc) -> Self {
        Self {
            content: string.to_string(),
            start: location.get_start(),
        }
    }
    fn len(&self) -> usize {
        self.content.len()
    }
    fn get_location(&self) -> Loc {
        Loc::new(self.start, self.end())
    }
    fn unsafe_new(string: &str) -> Self {
        Self {
            content: string.to_string(),
            start: Pos::new(0, 0),
        }
    }
    fn set_location_start(&mut self, start: Pos) {
        self.start = start;
    }
}

impl ConvertibleToString for &str {
    fn to_loc_string(&self) -> Str {
        Str::unsafe_new(self)
    }
}
