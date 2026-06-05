use crate::ast;

use crate::ast::Str;

mod generate;
mod target;

/// FIXME: remove expensive clones.
#[must_use]
pub fn generate_module(module: &ast::elaborated::Module<Str>) -> String {
    let module = ast::Module::from_elaborated(module);

    generate::generate_module(&module)
}
