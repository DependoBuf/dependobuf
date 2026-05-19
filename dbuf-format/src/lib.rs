//! Formatter, that pretty-prints CST representation of dbuf file

mod strategy;
mod utils;

use dbuf_core::cst::*;

use pretty::{BoxAllocator, DocAllocator};

/// Converts CST Tree to pretty printed String
///
/// # Panics
///
/// Panincs if found non UTF-8 symbols in `Tree`.
#[must_use]
pub fn pretty_print(t: &Tree) -> String {
    let alloc = &BoxAllocator;
    let mut write = Vec::new();

    let strategy_config = strategy::StrategyConfig { tab_size: 4 };
    let strategy = strategy::Strategy::new(strategy_config);

    let (_, doc) = utils::run(t, strategy, alloc);

    doc.append(alloc.hardline())
        .render(80, &mut write)
        .expect("ok");

    String::from_utf8(write).expect("printed code is utf8")
}
