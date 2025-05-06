//! Provides Handler trait, wich should be implemented by every handler.
//!

use tower_lsp::lsp_types::InitializeParams;
use tower_lsp::lsp_types::ServerCapabilities;

/// Capabilities of handler
pub trait Capabilities {
    fn apply(self, capabilities: &mut ServerCapabilities);
}

/// Handler trait, wich should be implemented by every handler.
pub trait Handler {
    fn new() -> Self;
    /// Returns capabilities, based on own functionality.
    fn init(&self, init: &InitializeParams) -> impl Capabilities;
}
