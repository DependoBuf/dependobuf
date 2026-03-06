//! Provides function, that returns Ast sample.
//!
//! TODO: remove such API when parsers are ready.
//!
//! Current sample:
//! ```dbuf
#![doc = include_str!("sample.dbuf")]
//! ```
//!

mod elaborated_ast_example;

use elaborated_ast_example::*;

use super::ast_access::ElaboratedAst;

pub fn default_elaborated_ast() -> ElaboratedAst {
    rename_elaborated_ast()
}

#[cfg(test)]
use super::ast_access::ParsedAst;
#[cfg(test)]
pub fn default_parsed_ast() -> ParsedAst {
    use dbuf_core::cst::convert_to_ast;
    use dbuf_core::cst::parse_to_cst;

    let code = include_str!("sample.dbuf");

    let (tree, err) = parse_to_cst(code);

    assert!(err.is_empty());
    assert!(tree.is_some());

    convert_to_ast(&tree.unwrap())
}
