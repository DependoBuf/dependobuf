use std::sync::Mutex;

use tower_lsp::lsp_types::{Position, Url};

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
    pub fn set(&self, doc: Url, version: i32, pos: Position, symbol: Symbol) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.document = Some(doc);
            cache.version = version;
            cache.pos = pos;
            cache.symbol = Some(symbol);
        }
    }

    pub fn get(&self, doc: &Url, version: i32, pos: Position) -> Option<Symbol> {
        if let Ok(mut cache) = self.cache.lock() {
            if let Some(url) = &cache.document {
                if url == doc && cache.version == version && cache.pos == pos {
                    cache.document.take();
                    return Some(cache.symbol.take().expect("setted with document"));
                }
            }
        }

        None
    }
}
