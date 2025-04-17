//! `textDocument/rename` and `textDocument/prepareRename` helpers.
//!

use dbuf_core::ast::elaborated::{Constructor, ConstructorNames};
use tower_lsp::jsonrpc::Result;

use crate::common::ast_access::{ElaboratedAst, ElaboratedHelper};
use crate::common::dbuf_language;
use crate::common::errors::bad_rename_error;
use crate::common::navigator::Symbol;

/// Check if symbol can be renamed.
pub fn renameable_symbol(symbol: &Symbol) -> bool {
    match symbol {
        Symbol::Type(t) => !dbuf_language::get_bultin_types().contains(t),
        Symbol::Dependency {
            t: _,
            dependency: _,
        } => true,
        Symbol::Field {
            constructor: _,
            field: _,
        } => true,
        Symbol::Constructor(_) => false,
        Symbol::None => false,
    }
}

/// Check if symbol can be renamed to new_name without conflicts.
pub fn renameable_to_symbol(symbol: &Symbol, new_name: &String, ast: &ElaboratedAst) -> Result<()> {
    if new_name.is_empty() {
        return Err(bad_rename_error("rename to empty string"));
    }
    if dbuf_language::get_bultin_types().contains(new_name) {
        return Err(bad_rename_error("rename to buildin type is forbidden"));
    }
    if dbuf_language::get_keywords().contains(new_name) {
        return Err(bad_rename_error("rename to keyword is forbidden"));
    }

    match symbol {
        Symbol::Type(t) => {
            if dbuf_language::get_bultin_types().contains(t) {
                return Err(bad_rename_error("buildin type can't be renamed"));
            }
            if !dbuf_language::is_correct_type_name(new_name) {
                return Err(bad_rename_error(
                    format!("'{}' is not correct type name", new_name).as_ref(),
                ));
            }
            if t == new_name {
                return Err(bad_rename_error("useless rename"));
            }
            if ast.has_type_or_constructor(new_name) {
                return Err(bad_rename_error(
                    format!("constructor or type '{}' exist", new_name).as_ref(),
                ));
            }
        }
        Symbol::Dependency {
            t: type_name,
            dependency: d,
        } => {
            if d == new_name {
                return Err(bad_rename_error("useless rename"));
            }
            if !dbuf_language::is_correct_dependency_name(new_name) {
                return Err(bad_rename_error(
                    format!("'{}' is not correct dependency name", new_name).as_ref(),
                ));
            }
            if !type_dependency_valid_rename(ast, type_name, new_name) {
                return Err(bad_rename_error(
                    format!("type '{}' already contains '{}'", type_name, new_name).as_ref(),
                ));
            }
        }
        Symbol::Field {
            constructor: ctr,
            field: f,
        } => {
            if f == new_name {
                return Err(bad_rename_error("useless rename"));
            }
            if !dbuf_language::is_correct_field_name(new_name) {
                return Err(bad_rename_error(
                    format!("'{}' is not correct field name", new_name).as_ref(),
                ));
            }
            if !constructor_field_valid_rename(ast, ctr, new_name) {
                return Err(bad_rename_error(
                    format!("constructor '{}' already contains '{}'", ctr, new_name).as_ref(),
                ));
            }
        }
        Symbol::Constructor(_) => {
            return Err(bad_rename_error("Constructors rename is not supported yet"))
        }
        Symbol::None => return Err(bad_rename_error("can't rename not symbol")),
    };

    Ok(())
}

/// Check if any type's dependency can be renamed to rename.
fn type_dependency_valid_rename(ast: &ElaboratedAst, type_name: &str, rename: &str) -> bool {
    if let Some(t) = ast.get_type(type_name) {
        if t.dependencies.iter().any(|d| d.0 == rename) {
            return false;
        }
        match &t.constructor_names {
            ConstructorNames::OfMessage(ctr) => {
                let ctr = ast.get_constructor(ctr).expect("valid ast");
                return !constructor_has_field(ctr, rename);
            }
            ConstructorNames::OfEnum(ctrs) => {
                return !ctrs.iter().any(|ctr| {
                    let ctr = ast.get_constructor(ctr).expect("valid ast");
                    constructor_has_field(ctr, rename)
                })
            }
        }
    }
    false
}

/// Check if any constructor's field can be renamed to rename.
fn constructor_field_valid_rename(
    ast: &ElaboratedAst,
    constructor_name: &str,
    rename: &str,
) -> bool {
    if let Some(type_name) = ast.get_constructor_type(constructor_name) {
        return type_dependency_valid_rename(ast, type_name, rename);
    }
    false
}

/// Check if constructor have field or implicit variable field.
fn constructor_has_field<T: AsRef<str>>(ctr: &Constructor<T>, field: &str) -> bool {
    if ctr.implicits.iter().any(|i| i.0.as_ref() == field) {
        return true;
    }
    if ctr.fields.iter().any(|f| f.0.as_ref() == field) {
        return true;
    }
    false
}
