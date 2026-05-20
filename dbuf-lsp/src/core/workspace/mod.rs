//! Helps with controlling access to ast.
//!
//! Exports ast types:
//! * `ParsedAst`.
//! * `ElaboratedAst`.
//!
//! Exports controll access:
//! * `WorkspaceAccess`.
//!

mod elaborated_ast;
mod file;
mod location;
mod parsed_ast;
mod parsers;
mod string;

use dashmap::DashMap;
use dashmap::mapref::one::Ref;

use dbuf_core::arena::InternedString;
use dbuf_core::cst;
use dbuf_core::location::LocatedName;
use dbuf_core::location::Offset;
use tower_lsp::lsp_types::Url;

use parsers::*;

pub use elaborated_ast::ElaboratedHelper;
pub use file::*;
pub use location::*;
pub use string::*;

/// String for `ParsedAst`
pub type Str = LocatedName<InternedString, Offset>;
/// Location for `ParsedAst`
pub type Loc = Location;
/// Alias for `cst::Tree`
pub type Cst = cst::Tree;
/// Alias for `ElaboratedAst`
pub use elaborated_ast::ElaboratedAst;
/// Alias for `ParsedAst`
pub use parsed_ast::ParsedAst;

/// Guards multicore access to files in workspace.
pub struct WorkspaceAccess {
    files: DashMap<Url, File>,
}

impl Default for WorkspaceAccess {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceAccess {
    #[must_use]
    pub fn new() -> WorkspaceAccess {
        WorkspaceAccess {
            files: DashMap::new(),
        }
    }

    /// Builds asts for text and setup File for it.
    pub fn open(&self, url: Url, version: i32, text: &str) {
        let mut errors = vec![];

        let cst = {
            let (cst, err) = get_cst(text);
            errors.extend(err);
            cst
        };

        let parsed = cst.as_ref().and_then(|t| {
            let (parsed, err) = get_parsed(t);
            errors.extend(err);
            parsed
        });

        let elaborated = parsed.as_ref().and_then(|t| {
            let (elaborated, err) = get_elaborated(t);
            errors.extend(err);
            elaborated
        });

        let file = File::new(url.clone(), version, cst, parsed, elaborated, errors);

        self.files.insert(url, file);
    }

    /// Builds asts for text and change File's asts.
    ///
    /// # Panics
    ///
    /// Will panic if version is not monotonic (old file version is higher than current).
    pub fn change(&self, url: &Url, version: i32, text: &str) {
        let mut errors = vec![];

        let cst = {
            let (cst, err) = get_cst(text);
            errors.extend(err);
            cst
        };

        let parsed = cst.as_ref().and_then(|t| {
            let (parsed, err) = get_parsed(t);
            errors.extend(err);
            parsed
        });

        let elaborated = parsed.as_ref().and_then(|t| {
            let (elaborated, err) = get_elaborated(t);
            errors.extend(err);
            elaborated
        });

        self.files.alter(&url.to_owned(), |_u, f| {
            f.modify(version, cst, parsed, elaborated, errors)
        });
    }

    /// Returns File by `url`.
    ///
    /// # Panics
    ///
    /// Will panic if `open` method is not called with `url`.
    #[must_use]
    pub fn read(&self, url: &Url) -> Ref<'_, Url, File> {
        self.files.get(url).expect("file should be opened")
    }

    /// Removes File from opened files.
    ///
    /// # Panics
    ///
    /// Will panic if `open` method is not called with `url`.
    pub fn close(&self, url: &Url) {
        self.files.remove(url).expect("file should be opened");
    }
}
