//! `textDocument/rename` and `textDocument/prepareRename` helpers.
//!

use dbuf_core::ast::elaborated::{Constructor, ConstructorNames};
use tower_lsp::jsonrpc::Result;

use crate::core::ast_access::{ElaboratedAst, ElaboratedHelper};
use crate::core::dbuf_language;
use crate::core::errors::rename_errors;
use crate::core::navigator::Symbol;

/// Check if symbol can be renamed.
///
/// TODO:
/// * renameable constructor.
/// * renameable alias.
pub fn renameable_symbol(symbol: &Symbol) -> bool {
    match symbol {
        Symbol::Type(t) => !dbuf_language::get_builtin_types().contains(t),
        Symbol::Dependency {
            t: _,
            dependency: _,
        } => true,
        Symbol::Field {
            constructor: _,
            field: _,
        } => true,
        Symbol::Alias {
            t: _,
            branch_id: _,
            name: _,
        } => false,
        Symbol::Constructor(_) => false,
        Symbol::None => false,
    }
}

/// Check if symbol can be renamed to new_name without conflicts.
pub fn renameable_to_symbol(symbol: &Symbol, new_name: &String, ast: &ElaboratedAst) -> Result<()> {
    if new_name.is_empty() {
        return rename_errors::rename_to_empty_error();
    }
    if dbuf_language::get_builtin_types().contains(new_name) {
        return rename_errors::rename_to_builtin_type_error();
    }
    if dbuf_language::get_keywords().contains(new_name) {
        return rename_errors::rename_to_keyword_error();
    }

    match symbol {
        Symbol::Type(t) => {
            if dbuf_language::get_builtin_types().contains(t) {
                return rename_errors::rename_of_builtin_type_error();
            }
            if !dbuf_language::is_correct_type_name(new_name) {
                return rename_errors::rename_to_bad_type_error(new_name);
            }
            if t == new_name {
                return rename_errors::rename_to_old_error();
            }
            if ast.has_type_or_constructor(new_name) {
                return rename_errors::rename_to_existing_type_error(new_name);
            }
        }
        Symbol::Dependency {
            t: type_name,
            dependency: d,
        } => {
            if d == new_name {
                return rename_errors::rename_to_old_error();
            }
            if !dbuf_language::is_correct_dependency_name(new_name) {
                return rename_errors::rename_to_bad_dependency_error(new_name);
            }
            if !type_dependency_valid_rename(ast, type_name, new_name) {
                return rename_errors::rename_to_existing_resource_error(type_name, new_name);
            }
        }
        Symbol::Field {
            constructor: ctr,
            field: f,
        } => {
            if f == new_name {
                return rename_errors::rename_to_old_error();
            }
            if !dbuf_language::is_correct_field_name(new_name) {
                return rename_errors::rename_to_bad_field_error(new_name);
            }
            let t = ast.get_constructor_type(ctr).expect("valid symbol");
            if !constructor_field_valid_rename(ast, t, ctr, new_name) {
                return rename_errors::rename_to_existing_resource_error(t, new_name);
            }
        }
        Symbol::Alias {
            t: _,
            branch_id: _,
            name: _,
        } => return rename_errors::rename_of_alias_error(),
        Symbol::Constructor(_) => return rename_errors::rename_of_constructor_error(),
        Symbol::None => return rename_errors::rename_none_symbol_error(),
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
    type_name: &str,
    _constructor_name: &str,
    rename: &str,
) -> bool {
    type_dependency_valid_rename(ast, type_name, rename)
}

/// Check if constructor have field or implicit variable field.
fn constructor_has_field<T: AsRef<str>>(ctr: &Constructor<T>, field: &str) -> bool {
    ctr.implicits
        .iter()
        .map(|i| &i.0)
        .chain(ctr.fields.iter().map(|i| &i.0))
        .any(|f| f.as_ref() == field)
}
