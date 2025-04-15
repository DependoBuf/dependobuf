use super::ElaboratedAst;
use super::ParsedAst;

use crate::common::default_ast::default_elaborated_ast;
use crate::common::default_ast::default_parsed_ast;

pub fn get_parsed(_text: &String) -> ParsedAst {
    default_parsed_ast()
}

pub fn get_elaborated(_text: &String) -> ElaboratedAst {
    default_elaborated_ast()
}
