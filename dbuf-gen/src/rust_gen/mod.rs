use crate::{ast::Module, format::BoxDoc, generate::GlobalContext};

use context::NamingContext;

mod context;
mod generate;
mod objects;
mod prelude;

pub fn generate_module<'a>(module: &Module, ctx: GlobalContext<'a>) -> BoxDoc<'a> {
    let mut namespace = NamingContext::root();
    module.generate((ctx, &mut namespace))
}
