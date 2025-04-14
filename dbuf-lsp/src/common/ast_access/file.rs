use std::collections::BTreeMap;

use super::ElaboratedAst;
use super::ParsedAst;

#[derive(Debug)]
pub struct File {
    version: i32,
    parsed_ast: ParsedAst,
    elaborated_ast: ElaboratedAst,
}

impl File {
    pub fn new() -> File {
        File {
            version: -1,
            parsed_ast: vec![],
            elaborated_ast: ElaboratedAst {
                types: vec![],
                constructors: BTreeMap::new(),
            },
        }
    }

    pub fn get_parsed(&self) -> &ParsedAst {
        &self.parsed_ast
    }
    pub fn get_elaborated(&self) -> &ElaboratedAst {
        &self.elaborated_ast
    }

    pub fn set_ast(
        &mut self,
        new_version: i32,
        new_parsed: ParsedAst,
        new_elaborated: ElaboratedAst,
    ) {
        assert!(self.version < new_version);
        self.version = new_version;
        self.parsed_ast = new_parsed;
        self.elaborated_ast = new_elaborated;
    }
}
