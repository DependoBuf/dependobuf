use tower_lsp::lsp_types::{Position, Url};

use tokio::sync::Mutex;

use crate::core::navigator::Symbol;

#[derive(Default)]
struct Inner {
    document: Option<Url>,
    version: i32,
    pos: Position,
    symbol: Option<Symbol>,
}

#[derive(Default)]
pub struct RenameCache {
    cache: Mutex<Inner>,
}

impl RenameCache {
    pub async fn set(&self, doc: Url, version: i32, pos: Position, symbol: Symbol) {
        let mut cache = self.cache.lock().await;
        cache.document = Some(doc);
        cache.version = version;
        cache.pos = pos;
        cache.symbol = Some(symbol);
    }

    pub async fn get(&self, doc: &Url, version: i32, pos: Position) -> Option<Symbol> {
        let mut cache = self.cache.lock().await;

        if let Some(url) = &cache.document
            && url == doc
            && cache.version == version
            && cache.pos == pos
        {
            cache.document.take();
            Some(cache.symbol.take().expect("setted with document"))
        } else {
            None
        }
    }
}
