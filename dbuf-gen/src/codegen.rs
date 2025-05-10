use std::io;

use crate::{ast, format::BoxAllocator, generate::GlobalContext, rust_gen};

pub fn generate_module<Writer: io::Write>(
    module: ast::elaborated::Module<String>,
    w: &mut Writer,
) -> io::Result<()> {
    let allocator = BoxAllocator;
    let ctx = GlobalContext { alloc: &allocator };
    let module = ast::Module::from_elaborated(module);
    let doc = rust_gen::generate_module(module, ctx);
    doc.render(40, w)
}
