//! Allows fix locations as if code was properly formatted
//!

use std::io;

use crate::common::pretty_printer::PrettyWriter;

use dbuf_core::ast::parsed::Module;
use dbuf_core::location::Location;

type Str = String;
type Loc = Location;

pub fn fix_locations(module: &mut Module<Loc, Str>) {
    let mut sink = io::Sink::default();
    let mut writer = PrettyWriter::new(&mut sink);
    writer
        .parse_module(module)
        .expect("module properly formatted");
}
