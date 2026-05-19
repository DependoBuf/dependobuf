//! Module contains entry point to run lsp.
use std::process::exit;

#[cfg(feature = "lsp")]
use dbuf_lsp::backend;

/// Main for LSP.
pub fn run() -> ! {
    #[cfg(feature = "lsp")]
    {
        backend::run();
        exit(0);
    }

    #[cfg(not(feature = "lsp"))]
    {
        eprintln!("LSP feature is not enabled");
        exit(1);
    }
}
