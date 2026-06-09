mod completion_tests;

use tower_lsp::lsp_types::InitializeParams;

use dbuf_lsp::completion::Handler;
use dbuf_lsp::handler_box::HandlerBox;

type HandlerType = HandlerBox<Handler>;

fn get_handler() -> HandlerBox<Handler> {
    let ans = HandlerBox::<Handler>::default();
    let _ = ans.init(&InitializeParams::default());
    ans
}
