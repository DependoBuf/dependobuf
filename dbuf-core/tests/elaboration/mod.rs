pub mod elaboration_correct;
pub mod elaboration_incorrect;

use dbuf_core::arena::InternedString;
use dbuf_core::ast::parsed;
use dbuf_core::cst::{convert_to_ast, parse_to_cst};
use dbuf_core::location::{LocatedName, Location, Offset};
use std::fs;
use std::path::Path;
type Loc = Location<Offset>;

type Name = LocatedName<InternedString, Offset>;

/// Returns the AST of the schema from the file at the specified path.
/// # Panics
/// If reading the input or parsing the CST was an error
#[must_use]
pub fn parse_file(path: &Path) -> parsed::Module<Loc, Name> {
    let input = fs::read_to_string(path).expect("file was read successfully");
    let (tree, errors) = parse_to_cst(&input);
    assert!(tree.is_some(), "CST parse failed for '{}'", path.display());
    assert!(errors.is_empty(), "Parse errors in '{}'", path.display());
    let tree = tree.expect("CST parsed successfully");
    convert_to_ast(&tree)
}
