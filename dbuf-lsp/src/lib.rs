//! TODO:
//! * `textDocument/InlayHint`
//! * common utils tests
//! * e2e server tests
//! * re-read files to fill documentation.
//! * add literals to navigator symbol.
//! * rewrite pretty printer.
//! * Each handler has it's own accesses

pub(crate) mod core;

pub use core::ast_access::WorkspaceAccess;

pub mod handler_box;

pub mod action;
pub mod completion;
pub mod diagnostic;
pub mod navigation;
