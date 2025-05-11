//! `textDocument/rename` and `textDocument/prepareRename` helpers.
//!

use dbuf_core::ast::elaborated::{Constructor, ConstructorNames};
use tower_lsp::jsonrpc::Result;

use crate::core::ast_access::{ElaboratedAst, ElaboratedHelper};
use crate::core::dbuf_language::{self, FieldName, TypeName};
use crate::core::errors::RenameError;
use crate::core::navigator::Symbol;

/// Check if symbol can be renamed.
///
/// TODO:
/// * renameable constructor.
/// * renameable alias.
pub fn renameable_symbol(symbol: &Symbol) -> bool {
    match symbol {
        Symbol::Type { type_name } => !dbuf_language::get_builtin_types().contains(type_name),
        Symbol::Dependency {
            type_name: _,
            dependency: _,
        } => true,
        Symbol::Field {
            type_name: _,
            constructor: _,
            field: _,
        } => true,
        Symbol::Alias {
            type_name: _,
            branch_id: _,
            alias: _,
        } => false,
        Symbol::Constructor {
            type_name: _,
            constructor: _,
        } => false,
        Symbol::None => false,
    }
}

/// Check if symbol can be renamed to new_name without conflicts.
pub fn renameable_to_symbol(symbol: &Symbol, new_name: &str, ast: &ElaboratedAst) -> Result<()> {
    if new_name.is_empty() {
        return RenameError::ToEmpty.into();
    }
    if dbuf_language::get_builtin_types().contains(new_name) {
        return RenameError::ToBuiltin.into();
    }
    if dbuf_language::get_keywords().contains(new_name) {
        return RenameError::ToKeyword.into();
    }

    match symbol {
        Symbol::Type { type_name } => {
            if type_name == new_name {
                return RenameError::ToPrevious.into();
            }
            if dbuf_language::get_builtin_types().contains(type_name) {
                return RenameError::OfBuiltin.into();
            }
            let new_type_name: TypeName = match new_name.try_into() {
                Ok(type_name) => type_name,
                Err(_) => return RenameError::ToBadType(new_name.to_owned()).into(),
            };
            let checker = ConflictChecker::new(ast);
            if checker.has_type_or_constructor(new_type_name) {
                return RenameError::ToExistingType(new_name.to_owned()).into();
            }
        }
        Symbol::Dependency {
            type_name,
            dependency,
        } => {
            if dependency == new_name {
                return RenameError::ToPrevious.into();
            }
            let new_dependency_name: FieldName = match new_name.try_into() {
                Ok(dependency) => dependency,
                Err(()) => return RenameError::ToBadDependency(new_name.to_owned()).into(),
            };
            let checker = ConflictChecker::new(ast);
            if checker.type_has_resourse(type_name, new_dependency_name) {
                return RenameError::ToExistingResource {
                    t: type_name.to_owned(),
                    r: new_name.to_owned(),
                }
                .into();
            }
        }
        Symbol::Field {
            type_name,
            constructor: _,
            field,
        } => {
            if field == new_name {
                return RenameError::ToPrevious.into();
            }
            let new_field_name: FieldName = match new_name.try_into() {
                Ok(field_name) => field_name,
                Err(_) => return RenameError::ToBadField(new_name.to_owned()).into(),
            };
            let checker = ConflictChecker::new(ast);
            if checker.type_has_resourse(type_name, new_field_name) {
                return RenameError::ToExistingResource {
                    t: type_name.to_owned(),
                    r: new_name.to_owned(),
                }
                .into();
            }
        }
        Symbol::Alias {
            type_name: _,
            branch_id: _,
            alias: _,
        } => return RenameError::OfAlias.into(),
        Symbol::Constructor {
            type_name: _,
            constructor: _,
        } => return RenameError::OfConstructor.into(),
        Symbol::None => return RenameError::OfNone.into(),
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
