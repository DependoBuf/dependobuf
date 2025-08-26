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
use dbuf_core::ast::parsed::located_name::LocatedName;
use dbuf_core::ast::parsed::location::Offset;
use tower_lsp::lsp_types::Url;

use parsers::*;

pub use elaborated_ast::ElaboratedHelper;
pub use file::*;
pub use location::*;
pub use string::*;

/// String for `ParsedAst`
pub type Str = LocatedName<String, Offset>;
/// Location for `ParsedAst`
pub type Loc = Location;
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
        let parsed: ParsedAst = get_parsed(text);
        let elaborated: ElaboratedAst = get_elaborated(text);

        let file = File::new(version, parsed, elaborated);

        self.files.insert(url, file);
    }

    /// Builds asts for text and change File's asts.
    ///
    /// # Panics
    ///
    /// Will panic if version is not monotonic (old file version is higher than current).
    pub fn change(&self, url: &Url, version: i32, text: &str) {
        let parsed: ParsedAst = get_parsed(text);
        let elaborated: ElaboratedAst = get_elaborated(text);

        let file = File::new(version, parsed, elaborated);

        let old = self
            .files
            .insert(url.to_owned(), file)
            .expect("file should be opened");

        assert!(old.get_version() < version, "versions shoud be monotonic");
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
