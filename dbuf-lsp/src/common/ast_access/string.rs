use std::fmt;

use super::location::*;

#[derive(Debug, Clone)]
pub struct LocString {
    string: String,
    location: Location,
}

pub trait ConvertibleToString {
    fn to_loc_string(&self) -> LocString;
}

impl LocString {
    pub fn new(string: &str) -> LocString {
        LocString {
            string: string.to_string(),
            location: Location::new_empty(),
        }
    }
    pub fn len(&self) -> usize {
        self.string.len()
    }
    pub fn set_location_start(&mut self, start: Position) {
        self.location.start = start;
    }
    pub fn set_location_end(&mut self, end: Position) {
        self.location.end = end;
    }
}

impl AsRef<str> for LocString {
    fn as_ref(&self) -> &str {
        return &self.string;
    }
}

impl fmt::Display for LocString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.string.fmt(f)
    }
}

impl ConvertibleToString for &str {
    fn to_loc_string(&self) -> LocString {
        LocString::new(&self)
    }
}
