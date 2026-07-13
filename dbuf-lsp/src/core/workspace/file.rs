//! Module exports struct `File` - representation of one file in workspace.
//!

use std::sync::Arc;

use tower_lsp::lsp_types::Url;

use crate::core::errors::Error;

use super::Cst;
use super::ElaboratedAst;
use super::ParsedAst;

/// Represents one file in workspace. Contains its version, and asts.
#[derive(Debug)]
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
/// last success version.
///
/// Type params:
/// T - type of stored object T,
/// U - type of stored link to File object.
#[derive(Debug, Clone)]
pub enum MaybeOutdated<T, U> {
    /// Correspond to last version of file
    Current(T),
    /// Correspond to one of previous versions of file
    ///
    /// #0: Stored object
    ///
    /// #1: Correspong file version, when object was created
    Outdated(T, U),
    /// Never constructed
    Nothing,
}

/// Storage type for file.
type Saved<T> = MaybeOutdated<Arc<T>, Arc<File>>;

/// Read access type for file.
pub type Stored<'a, T> = MaybeOutdated<&'a T, &'a File>;

impl<T> Saved<T> {
    fn new(value: Option<T>) -> Self {
        value.map(Arc::new).map_or(Self::Nothing, Self::Current)
    }

    fn outdate(&self, old_version: Arc<File>) -> Self {
        match self {
            Self::Current(v) => Self::Outdated(v.clone(), old_version),
            Self::Outdated(v, version) => Self::Outdated(v.clone(), version.clone()),
            Self::Nothing => Self::Nothing,
        }
    }

    fn modify(&self, value: Saved<T>, old_version: Arc<File>) -> Self {
        match value {
            Self::Nothing => self.outdate(old_version),
            _ => value,
        }
    }

    fn to_stored(&self) -> Stored<'_, T> {
        match self {
            MaybeOutdated::Current(t) => MaybeOutdated::Current(t),
            MaybeOutdated::Outdated(t, v) => MaybeOutdated::Outdated(t, v),
            MaybeOutdated::Nothing => MaybeOutdated::Nothing,
        }
    }
}

impl<'a, T> Stored<'a, T> {
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

    pub(super) fn modify(self: Arc<Self>, newer: File) -> File {
        assert!(self.version < newer.version);
        let old = self.clone();
        File {
            uri: newer.uri,
            version: newer.version,
            cst: self.cst.modify(newer.cst, old.clone()),
            parsed_ast: self.parsed_ast.modify(newer.parsed_ast, old.clone()),
            elaborated_ast: self.elaborated_ast.modify(newer.elaborated_ast, old),
            errors: newer.errors,
        }
    }

    pub fn get_uri(&self) -> &Url {
        &self.uri
    }

    pub fn get_cst(&self) -> Stored<'_, Cst> {
        self.cst.to_stored()
    }

    pub fn get_parsed(&self) -> Stored<'_, ParsedAst> {
        self.parsed_ast.to_stored()
    }

    pub fn get_elaborated(&self) -> Stored<'_, ElaboratedAst> {
        self.elaborated_ast.to_stored()
    }

    pub fn get_version(&self) -> i32 {
        assert!(self.version != -1);
        self.version
    }

    pub fn get_errors(&self) -> &Vec<Error> {
        &self.errors
    }
}
