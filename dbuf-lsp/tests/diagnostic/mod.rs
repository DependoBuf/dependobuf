mod lens_tests;
mod references_tests;
mod semantic_tests;
mod symbol_tests;

use tower_lsp::lsp_types::InitializeParams;

use dbuf_lsp::diagnostic::Handler;
use dbuf_lsp::handler_box::HandlerBox;

fn get_handler() -> HandlerBox<Handler> {
    let ans = HandlerBox::<Handler>::default();
    let _ = ans.init(&InitializeParams::default());
    ans
}
