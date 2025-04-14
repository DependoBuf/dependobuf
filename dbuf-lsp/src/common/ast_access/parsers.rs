use std::collections::BTreeMap;

use super::ElaboratedAst;
use super::ParsedAst;

use crate::common::default_ast::default_ast;

pub fn get_parsed(_: &String) -> ParsedAst {
    default_ast()
}

pub fn get_elaborated(_: &String) -> ElaboratedAst {
    ElaboratedAst {
        types: vec![],
        constructors: BTreeMap::new(),
    }
}
