mod format_tests;
mod rename_tests;

use tower_lsp::lsp_types::InitializeParams;

use super::Handler;
use crate::handler_box::HandlerBox;

fn get_handler() -> HandlerBox<Handler> {
    let ans = HandlerBox::<Handler>::default();
    let _ = ans.init(&InitializeParams::default());
    ans
}
