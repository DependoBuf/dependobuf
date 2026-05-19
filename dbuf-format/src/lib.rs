//! Module exports `PrettyPrinter`, that can pretty-print CST representation of dbuf file.
mod strategy;
mod utils;

use dbuf_core::cst::*;

use pretty::{BoxAllocator, DocAllocator};

/// Configurable pretty printer.
#[derive(Clone, Copy)]
pub struct PrettyPrinter {
    tab_size: usize,
}

impl Default for PrettyPrinter {
    fn default() -> Self {
        Self { tab_size: 4 }
    }
}

impl PrettyPrinter {
    #[must_use]
    pub fn with_tab_size(mut self, tab_size: usize) -> Self {
        self.tab_size = tab_size;
        self
    }

    /// Converts CST Tree to pretty printed String.
    ///
    /// # Panics
    /// Panics never.
    #[must_use]
    pub fn pretty_print(self, t: &Tree) -> String {
        let alloc = &BoxAllocator;
        let mut write = Vec::new();

        let strategy_config = strategy::StrategyConfig {
            tab_size: self.tab_size,
        };
        let strategy = strategy::Strategy::new(strategy_config);

        let (_, doc) = utils::run(t, strategy, alloc);

        doc.append(alloc.hardline())
            .render(80, &mut write)
            .expect("ok");

        String::from_utf8_lossy(&write).to_string()
    }
}
