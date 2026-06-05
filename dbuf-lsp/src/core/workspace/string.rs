//! Module exports
//! * `ConvertibleToString` trait, wich allows any type conversation to `LocatedName`.
//! * `LocNameHelper` trait with helpfull funtions for `LocatedName`.
//!

use super::location::LocationHelper;

use super::{Loc, Name};
use tower_lsp::lsp_types;

/// Helpers for `dbuf-core::LocatedName`.
pub trait LocNameHelper {
    /// Returns string's location.
    fn get_location(&self) -> Loc;
    /// Returns if positions in string's location.
    fn contains(&self, p: lsp_types::Position) -> bool {
        self.get_location().contains(p)
    }
}

impl LocNameHelper for Name {
    fn get_location(&self) -> Loc {
        Loc::new(self.start, self.end())
    }
}
