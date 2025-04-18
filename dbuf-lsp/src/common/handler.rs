//! Provides Handler trait, wich should be implemented by every handler.
//!

use tower_lsp::lsp_types::InitializeParams;
use tower_lsp::lsp_types::ServerCapabilities;
use tower_lsp::Client;

/// Handler trait, wich should be implemented by every handler.
pub trait Handler {
    fn new(client: Client) -> Self;
    /// Modifies capabilites, based on own functionality.
    fn init(&self, init: &InitializeParams, capabilites: &mut ServerCapabilities);
}
