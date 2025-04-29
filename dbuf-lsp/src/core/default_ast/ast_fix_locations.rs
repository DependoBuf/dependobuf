//! Allows fix locations as if code was properly formatted
//!

use core::fmt::Write;

use super::mutable_pretty_printer::PrettyPrinter;

use dbuf_core::ast::parsed::Module;

use crate::core::ast_access::{Loc, Str};

struct Sink;

impl Write for Sink {
    fn write_str(&mut self, _: &str) -> std::fmt::Result {
        Ok(())
    }
}

pub fn fix_locations(module: &mut Module<Loc, Str>) {
    let mut sink = Sink {};
    let mut writer = PrettyPrinter::new(&mut sink);
    writer
        .parse_module(module)
        .expect("module properly formatted");
}
