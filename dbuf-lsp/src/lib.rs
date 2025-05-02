//! TODO:
//! * `textDocument/InlayHint`
//! * common utils tests
//! * e2e server tests
//! * re-read files to fill documentation.
//! * add literals to navigator symbol.
//! * rewrite pretty printer.
//!

pub(crate) mod core;

pub use core::ast_access::WorkspaceAccess;

pub mod handler;

pub mod action_handler;
pub mod completion_handler;
pub mod diagnostic_handler;
pub mod navigation_handler;
