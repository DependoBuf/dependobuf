//! Module exports struct `File` - representation of one file in workspace.
//!

use crate::core::errors::Error;

use super::Cst;
use super::ElaboratedAst;
use super::ParsedAst;

/// Represents one file in workspace. Contains its version, and asts.
pub struct File {
    /// File's version.
    version: i32,
    /// Builded `cst::Tree`.
    cst: Saved<Cst>,
    /// Builded `ParsedAst`.
    parsed_ast: Saved<ParsedAst>,
    /// Builded `ElaboratedAst` for current version.
    elaborated_ast: Saved<ElaboratedAst>,
    /// Errors produced on parsing and elaborating,
    errors: Vec<Error>,
}

/// Structure that stores current version or
/// last success version
#[derive(Debug)]
pub enum Saved<T> {
    /// Correspond to last version of file
    Current(T),
    /// Correspond to one of previous versions of file
    Outdated(T, i32),
    /// Never constructed
    Nothing,
}

impl<T> Saved<T> {
    fn new(value: Option<T>) -> Self {
        value.map_or(Self::Nothing, Self::Current)
    }

    fn outdate(self, old_version: i32) -> Self {
        match self {
            Self::Current(v) => Self::Outdated(v, old_version),
            _ => self,
        }
    }

    fn modify(self, value: Option<T>, old_version: i32) -> Self {
        value.map_or_else(|| self.outdate(old_version), Self::Current)
    }

    fn get(&self) -> Saved<&T> {
        match self {
            Saved::Current(t) => Saved::Current(t),
            Saved::Outdated(t, i) => Saved::Outdated(t, *i),
            Saved::Nothing => Saved::Nothing,
        }
    }

    pub fn take(self) -> Option<T> {
        match self {
            Saved::Current(t) => Some(t),
            Saved::Outdated(t, _) => Some(t),
            Saved::Nothing => None,
        }
    }
}

impl File {
    pub(super) fn new(
        version: i32,
        cst: Option<Cst>,
        parsed: Option<ParsedAst>,
        elaborated: Option<ElaboratedAst>,
        errors: Vec<Error>,
    ) -> File {
        File {
            version,
            cst: Saved::new(cst),
            parsed_ast: Saved::new(parsed),
            elaborated_ast: Saved::new(elaborated),
            errors,
        }
    }

    pub(super) fn modify(
        self,
        new_version: i32,
        cst: Option<Cst>,
        parsed: Option<ParsedAst>,
        elaborated: Option<ElaboratedAst>,
        errors: Vec<Error>,
    ) -> File {
        assert!(self.version < new_version);
        File {
            version: new_version,
            cst: self.cst.modify(cst, self.version),
            parsed_ast: self.parsed_ast.modify(parsed, self.version),
            elaborated_ast: self.elaborated_ast.modify(elaborated, self.version),
            errors,
        }
    }

    pub fn get_cst(&self) -> Saved<&Cst> {
        self.cst.get()
    }

    pub fn get_parsed(&self) -> Saved<&ParsedAst> {
        self.parsed_ast.get()
    }

    pub fn get_elaborated(&self) -> Saved<&ElaboratedAst> {
        self.elaborated_ast.get()
    }

    pub fn get_version(&self) -> i32 {
        assert!(self.version != -1);
        self.version
    }

    pub fn get_errors(&self) -> &Vec<Error> {
        &self.errors
    }
}
