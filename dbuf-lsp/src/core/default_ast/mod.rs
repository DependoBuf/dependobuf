//! Provides function, that returns Ast sample.
//!
//! TODO: remove such API when parsers are ready.
//!

mod ast_builder;
mod ast_fix_locations;
mod mutable_pretty_printer;

mod elaborated_ast_example;
mod parsed_ast_example;

use elaborated_ast_example::*;
use parsed_ast_example::*;

use super::ast_access::{ElaboratedAst, ParsedAst};

pub fn default_parsed_ast() -> ParsedAst {
    rename_parsed_ast()
}

pub fn default_elaborated_ast() -> ElaboratedAst {
    rename_elaborated_ast()
}
