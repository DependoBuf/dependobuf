use tower_lsp::lsp_types::InitializeParams;
use tower_lsp::lsp_types::ServerCapabilities;

pub trait Handler {
    fn init(&self, init: InitializeParams, capabilites: &mut ServerCapabilities);
}
