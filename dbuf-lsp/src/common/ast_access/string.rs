//! Module exports
//! * ConvertibleToString trait, wich allows any type conversation to LocString.
//! * LocStringHelper trait with helpfull funtions for LocString.
//! * 'temporary' LocString, since there is no one in dbuf-core.
//!

use std::fmt;

use super::location::*;

/// String, containing location.
#[derive(Debug, Clone)]
pub struct LocString {
    string: String,
    location: Location,
}

/// Trait for types, that can be converted to LocString.
pub trait ConvertibleToString {
    fn to_loc_string(&self) -> LocString;
}

/// Helpers for dbuf-core::LocString (in future).
pub trait LocStringHelper {
    /// Constructs LocString with empty locations by &str.
    fn new(string: &str) -> Self;
    /// Checks if string is empty.
    fn is_empty(&self) -> bool;
    /// Returns string's len.
    fn len(&self) -> usize;
    /// Returns string's location.
    fn get_location(&self) -> Location;
    /// Sets location.
    fn set_location(&mut self, location: Location);
    /// Sets begin of string's location.
    fn set_location_start(&mut self, start: Position);
    /// Sets end of string's location.
    fn set_location_end(&mut self, end: Position);
}

impl LocStringHelper for LocString {
    fn new(string: &str) -> LocString {
        LocString {
            string: string.to_string(),
            location: Location::new_empty(),
        }
    }
    fn is_empty(&self) -> bool {
        self.string.is_empty()
    }
    fn len(&self) -> usize {
        self.string.len()
    }
    fn get_location(&self) -> Location {
        self.location
    }
    fn set_location(&mut self, location: Location) {
        self.location = location;
    }
    fn set_location_start(&mut self, start: Position) {
        self.location.start = start;
    }
    fn set_location_end(&mut self, end: Position) {
        self.location.end = end;
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
        LocString::new(self)
    }
}
