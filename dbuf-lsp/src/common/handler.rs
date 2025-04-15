use std::sync::Arc;

use tower_lsp::lsp_types::InitializeParams;
use tower_lsp::lsp_types::ServerCapabilities;
use tower_lsp::Client;

pub trait Handler {
    fn new(client: Arc<Client>) -> Self;
    fn init(&self, init: &InitializeParams, capabilites: &mut ServerCapabilities);
}
