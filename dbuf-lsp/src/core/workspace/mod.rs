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

use std::sync::Arc;

use dashmap::DashMap;

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

/// String type
pub type Str = InternedString;

/// String for `ParsedAst`
pub type Name = LocatedName<Str, Offset>;
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
    files: DashMap<Url, Arc<File>>,
}

impl Default for WorkspaceAccess {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "passing by value, so the task can be run in separate thread.
    Another option: use &'static and unsafe magic to convert &'a str to &'static str, since compile
    guarantees are preserved (since the string is on stack while the task is executing)"
)]
fn get_file(url: Url, version: i32, text: String) -> File {
    let mut errors = vec![];

    let (cst, err) = get_cst(&text);
    errors.extend(err);

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

    File::new(url, version, cst, parsed, elaborated, errors)
}

impl WorkspaceAccess {
    #[must_use]
    pub fn new() -> WorkspaceAccess {
        WorkspaceAccess {
            files: DashMap::new(),
        }
    }

    /// Builds asts for text and setup File for it.
    ///
    /// # Panics
    ///
    /// When can join a task from blocking task pool (never?).
    pub async fn open(&self, url: Url, version: i32, text: String) {
        let url_cloned = url.clone();

        let file = tokio::task::spawn_blocking(move || get_file(url, version, text))
            .await
            .unwrap();

        self.files.insert(url_cloned, Arc::new(file));
    }

    /// Builds asts for text and change File's asts.
    ///
    /// # Panics
    ///
    /// Will panic if version is not monotonic (old file version is higher than current).
    /// Also panics if cant join a task from blocking thread pool (never?).
    pub async fn change(&self, url: Url, version: i32, text: String) {
        let url_cloned = url.clone();
        let file = tokio::task::spawn_blocking(move || get_file(url, version, text))
            .await
            .unwrap();

        self.files.alter(&url_cloned, |_u, f| f.modify(file).into());
    }

    /// Returns File by `url`.
    ///
    /// # Panics
    ///
    /// Will panic if `open` method is not called with `url`.
    #[must_use]
    pub fn read(&self, url: &Url) -> Arc<File> {
        self.files.get(url).expect("file should be opened").clone()
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
