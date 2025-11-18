#[cfg(feature = "lsp")]
use dbuf_lsp::backend;

pub fn run() {
    #[cfg(feature = "lsp")]
    backend::run();

    #[cfg(not(feature = "lsp"))]
    eprintln!("LSP feature is not enabled");
}
