use crate::ast;

mod generate;

/// Generate Swift source code for the provided elaborated module.
///
/// The implementation is intentionally *very* small – it is **only** capable
/// of generating code for the subset of DependoBuf that is currently covered
/// by the canonicalisation tests (`basic` and `nat_vec`).
///
/// It is good enough for snapshot-testing purposes and can be gradually
/// replaced by a full-featured backend later.
#[must_use]
pub fn generate_module(module: ast::elaborated::Module<String>) -> String {
    // Convert the elaborated AST used by the type-checker into the internal
    // representation expected by the generators.
    let module = ast::Module::from_elaborated(module);
    generate::generate_module(&module)
}
