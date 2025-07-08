//! Module exports
//! * `ConvertibleToString` trait, wich allows any type conversation to `LocString`.
//! * `LocStringHelper` trait with helpfull funtions for `LocString`.
//! * 'temporary' `LocString`, since there is no one in dbuf-core.
//!

use std::fmt;

use super::location::*;

use tower_lsp::lsp_types;

/// String, containing location.
/// TODO: migrate to type in dbuf-core.
#[derive(Debug, Clone)]
pub struct LocString {
    string: String,
    location: Location,
}

/// Trait for types, that can be converted to `LocString`.
/// TODO: remove such API.
pub trait ConvertibleToString {
    fn to_loc_string(&self) -> LocString;
}

/// Helpers for `dbuf-core::LocString` (in future).
pub trait LocStringHelper {
    /// Constructs `LocString`.
    fn new(string: &str, location: Location) -> Self;
    /// Returns string's len.
    fn len(&self) -> usize;
    /// Returns string's location.
    fn get_location(&self) -> Location;
    /// Returns if positions in string's location.
    fn contains(&self, p: lsp_types::Position) -> bool {
        self.get_location().contains(p)
    }
    /// Constructs `LocString` with empty location.
    /// TODO: remove such API.
    fn unsafe_new(string: &str) -> Self;
    /// Sets begin of string's location.
    /// TODO: remove such API.
    fn set_location_start(&mut self, start: Position);
    /// Sets end of string's location.
    /// TODO: remove such API.
    fn set_location_end(&mut self, end: Position);
}

impl LocStringHelper for LocString {
    fn new(string: &str, location: Location) -> LocString {
        LocString {
            string: string.to_string(),
            location,
        }
    }
    fn len(&self) -> usize {
        self.string.len()
    }
    fn get_location(&self) -> Location {
        self.location
    }
    fn unsafe_new(string: &str) -> LocString {
        LocString {
            string: string.to_string(),
            location: Location::new_empty(),
        }
    }
    fn set_location_start(&mut self, start: Position) {
        self.location.reset_start(start);
    }
    fn set_location_end(&mut self, end: Position) {
        self.location.set_end(end);
    }
}

impl AsRef<str> for LocString {
    fn as_ref(&self) -> &str {
        &self.string
    }
}

impl fmt::Display for LocString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.string.fmt(f)
    }
}

impl ConvertibleToString for &str {
    fn to_loc_string(&self) -> LocString {
        LocString::unsafe_new(self)
    }
}
