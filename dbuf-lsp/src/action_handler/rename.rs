//! `textDocument/rename` and `textDocument/prepareRename` helpers.
//!

use dbuf_core::ast::elaborated::{Constructor, ConstructorNames};
use tower_lsp::jsonrpc::Result;

use crate::core::ast_access::{ElaboratedAst, ElaboratedHelper};
use crate::core::dbuf_language::{self, FieldName, TypeName};
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
pub fn renameable_to_symbol(symbol: &Symbol, new_name: &str, ast: &ElaboratedAst) -> Result<()> {
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
            if t == new_name {
                return rename_errors::rename_to_old_error();
            }
            if dbuf_language::get_builtin_types().contains(t) {
                return rename_errors::rename_of_builtin_type_error();
            }
            let new_type_name: TypeName = match new_name.try_into() {
                Ok(type_name) => type_name,
                Err(_) => return rename_errors::rename_to_bad_type_error(new_name),
            };
            let checker = ConflictChecker::new(ast);
            if checker.has_type_or_constructor(new_type_name) {
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
            let new_dependency_name: FieldName = match new_name.try_into() {
                Ok(dependency) => dependency,
                Err(()) => return rename_errors::rename_to_bad_dependency_error(new_name),
            };
            let checker = ConflictChecker::new(ast);
            if checker.type_has_resourse(type_name, new_dependency_name) {
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
            let new_field_name: FieldName = match new_name.try_into() {
                Ok(field_name) => field_name,
                Err(_) => return rename_errors::rename_to_bad_field_error(new_name),
            };
            let t = ast.get_constructor_type(ctr).expect("valid symbol");
            let checker = ConflictChecker::new(ast);
            if checker.type_has_resourse(t, new_field_name) {
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

struct ConflictChecker<'a> {
    ast: &'a ElaboratedAst,
}

impl ConflictChecker<'_> {
    fn new(ast: &ElaboratedAst) -> ConflictChecker<'_> {
        ConflictChecker { ast }
    }
    /// Checks if ast has type t.
    fn has_type_or_constructor(&self, t: TypeName) -> bool {
        self.ast.has_type_or_constructor(t.get())
    }
    /// Checks if type t_name has field/dependency/alias r
    fn type_has_resourse(&self, t_name: &str, r: FieldName) -> bool {
        let t = match self.ast.get_type(t_name) {
            Some(t) => t,
            None => return false,
        };
        if t.dependencies.iter().any(|d| d.0 == r.get()) {
            return true;
        }
        match &t.constructor_names {
            ConstructorNames::OfMessage(ctr) => {
                let ctr = self.ast.get_constructor(ctr).expect("valid ast");
                self.constructor_has_resourse(ctr, r)
            }
            ConstructorNames::OfEnum(ctrs) => ctrs.iter().any(|ctr| {
                let ctr = self.ast.get_constructor(ctr).expect("valid ast");
                self.constructor_has_resourse(ctr, r)
            }),
        }
    }
    /// Checks if constructor ctr has field/implicit r
    fn constructor_has_resourse<T: AsRef<str>>(&self, ctr: &Constructor<T>, r: FieldName) -> bool {
        ctr.implicits
            .iter()
            .map(|i| &i.0)
            .chain(ctr.fields.iter().map(|i| &i.0))
            .any(|f| f.as_ref() == r.get())
    }
}
