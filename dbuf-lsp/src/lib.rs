//! TODO:
//! * `textDocument/InlayHint`
//! * remove async/await -- clients log are useless (eprintln! is enough for debug, no need in other logs)
//! * common utils tests
//! * e2e server tests
//! * overview
//! * re-read files to fill documentation.
//! 
pub mod common;

pub mod action_handler;
pub mod completion_handler;
pub mod diagnostic_handler;
pub mod navigation_handler;
