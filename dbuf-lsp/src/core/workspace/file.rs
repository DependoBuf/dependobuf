//! Module exports struct `File` - representation of one file in workspace.
//!

use std::sync::Arc;

use tower_lsp::lsp_types::Url;

use crate::core::errors::Error;

use super::Cst;
use super::ElaboratedAst;
use super::ParsedAst;

/// Represents one file in workspace. Contains its version, and asts.
pub struct File {
    /// File location.
    uri: Url,
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
enum Saved<T> {
    /// Correspond to last version of file
    Current(Arc<T>),
    /// Correspond to one of previous versions of file
    Outdated(Arc<T>, i32),
    /// Never constructed
    Nothing,
}

impl<T> Clone for Saved<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Current(arg0) => Self::Current(arg0.clone()),
            Self::Outdated(arg0, arg1) => Self::Outdated(arg0.clone(), *arg1),
            Self::Nothing => Self::Nothing,
        }
    }
}

impl<T> Saved<T> {
    fn new(value: Option<T>) -> Self {
        value.map(Arc::new).map_or(Self::Nothing, Self::Current)
    }

    fn outdate(&self, old_version: i32) -> Self {
        match self {
            Self::Current(v) => Self::Outdated(v.clone(), old_version),
            Self::Outdated(v, version) => Self::Outdated(v.clone(), *version),
            Self::Nothing => Self::Nothing,
        }
    }

    fn modify(&self, value: Saved<T>, old_version: i32) -> Self {
        match value {
            Self::Nothing => self.outdate(old_version),
            _ => value,
        }
    }
}

/// Structure that contains reference to stored data.
/// Semantic is same as Saved, and lifetime is same as
/// an object, containing Saved.
pub enum Stored<'a, T> {
    Current(&'a T),
    Outdated(&'a T, i32),
    Nothing,
}

impl<'a, T> Stored<'a, T> {
    fn from_saved(saved: &'a Saved<T>) -> Self {
        match saved {
            Saved::Current(t) => Self::Current(t),
            Saved::Outdated(t, v) => Self::Outdated(t, *v),
            Saved::Nothing => Self::Nothing,
        }
    }

    pub fn take(self) -> Option<&'a T> {
        match self {
            Stored::Current(t) => Some(t),
            Stored::Outdated(t, _) => Some(t),
            Stored::Nothing => None,
        }
    }
}

impl File {
    pub(super) fn new(
        uri: Url,
        version: i32,
        cst: Option<Cst>,
        parsed: Option<ParsedAst>,
        elaborated: Option<ElaboratedAst>,
        errors: Vec<Error>,
    ) -> File {
        File {
            uri,
            version,
            cst: Saved::new(cst),
            parsed_ast: Saved::new(parsed),
            elaborated_ast: Saved::new(elaborated),
            errors,
        }
    }

    pub(super) fn modify(&self, newer: File) -> File {
        assert!(self.version < newer.version);
        File {
            uri: newer.uri,
            version: newer.version,
            cst: self.cst.modify(newer.cst, self.version),
            parsed_ast: self.parsed_ast.modify(newer.parsed_ast, self.version),
            elaborated_ast: self
                .elaborated_ast
                .modify(newer.elaborated_ast, self.version),
            errors: newer.errors,
        }
    }

    pub fn get_uri(&self) -> &Url {
        &self.uri
    }

    pub fn get_cst(&self) -> Stored<'_, Cst> {
        Stored::from_saved(&self.cst)
    }

    pub fn get_parsed(&self) -> Stored<'_, ParsedAst> {
        Stored::from_saved(&self.parsed_ast)
    }

    pub fn get_elaborated(&self) -> Stored<'_, ElaboratedAst> {
        Stored::from_saved(&self.elaborated_ast)
    }

    pub fn get_version(&self) -> i32 {
        assert!(self.version != -1);
        self.version
    }

    pub fn get_errors(&self) -> &Vec<Error> {
        &self.errors
    }
}
