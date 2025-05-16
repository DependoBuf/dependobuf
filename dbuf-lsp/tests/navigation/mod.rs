mod goto_tests;
mod hover_tests;
mod inlay_tests;

use tower_lsp::lsp_types::InitializeParams;

use dbuf_lsp::handler_box::HandlerBox;
use dbuf_lsp::navigation::Handler;

fn get_handler() -> HandlerBox<Handler> {
    let ans = HandlerBox::<Handler>::default();
    let _ = ans.init(&InitializeParams::default());
    ans
}
