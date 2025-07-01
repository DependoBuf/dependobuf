use crate::ast;

mod generate;
mod target;

pub fn generate_module(module: ast::elaborated::Module<String>) -> String {
    let module = ast::Module::from_elaborated(module);

    generate::generate_module(module)
}
