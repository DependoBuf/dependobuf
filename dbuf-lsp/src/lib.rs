//! Also there are (after line 1136 in tower-lsp::lib.rs) some workspace commands, which are useless while we have no module system
//!

pub mod ast_access;
pub mod common;

pub mod action_handler;
pub mod completion_handler;
pub mod diagnostic_handler;
pub mod navigation_handler;
