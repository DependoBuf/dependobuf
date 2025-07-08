//! Module exports struct `File` - representation of one file in workspace.
//!

use super::ElaboratedAst;
use super::ParsedAst;

/// Represents one file in workspace. Contains its version, and asts.
pub struct File {
    /// File's version.
    version: i32,
    /// Builded `ParsedAst`.
    parsed_ast: ParsedAst,
    /// Builded `ElaboratedAst`.
    elaborated_ast: ElaboratedAst,
}

impl File {
    pub(super) fn new(version: i32, parsed: ParsedAst, elaborated: ElaboratedAst) -> File {
        File {
            version,
            parsed_ast: parsed,
            elaborated_ast: elaborated,
        }
    }

    pub fn get_parsed(&self) -> &ParsedAst {
        assert!(self.version != -1);
        &self.parsed_ast
    }
    pub fn get_elaborated(&self) -> &ElaboratedAst {
        assert!(self.version != -1);
        &self.elaborated_ast
    }
    pub fn get_version(&self) -> i32 {
        assert!(self.version != -1);
        self.version
    }
}
