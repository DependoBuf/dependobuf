//! TODO:
//! * re-read files to fill documentation.
//! * add literals to navigator symbol.
//! * rewrite pretty printer.

pub(crate) mod core;

pub use core::workspace::WorkspaceAccess;

pub mod handler_box;

pub mod action;
pub mod completion;
pub mod diagnostic;
pub mod navigation;

pub mod backend;
