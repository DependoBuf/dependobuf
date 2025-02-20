//! Some methods do something i can't understand, so i can't find good place to them:
//! * `textDocument/moniker`
//!
//! Also there are (after line 1136 in tower-lsp::lib.rs) some workspace commands, which are useless while we have no module system
//!

pub mod ast_access;

pub mod action_handler;
pub mod completion_handler;
pub mod diagnostic_handler;
pub mod navigation_handler;
