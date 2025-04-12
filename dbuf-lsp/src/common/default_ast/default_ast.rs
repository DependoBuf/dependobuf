//! Provides function, that returns Ast sample.
//!

use super::ast_builder::AstBuilder;
use crate::common::ast_access::Ast;

use dbuf_core::ast::operators::*;
use dbuf_core::ast::parsed::*;

pub fn default_ast() -> Ast {
    let mut builder = AstBuilder::new();

    builder
        .with_message("Example")
        .with_dependency("d1", "String")
        .with_field("f1", "Int")
        .with_field("f2", "Int")
        .with_field("f3", "Int");

    builder.with_message("Empty");

    let e = builder
        .with_enum("EnumExample")
        .with_dependency("d1", "Int");

    let b1 = e
        .with_branch()
        .with_pattern(PatternNode::Literal(Literal::Int(1)));
    b1.with_constructor("Ctr1").with_field("f1", "Str");
    b1.with_constructor("Ctr2");
    b1.with_constructor("Ctr3")
        .with_field("f2", "String")
        .with_field("f3", "Bool");
    let b2 = e.with_branch().with_pattern(PatternNode::Underscore);
    b2.with_constructor("Ct");

    builder.construct()
}
