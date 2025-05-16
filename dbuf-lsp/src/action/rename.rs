//! `textDocument/rename` and `textDocument/prepareRename` helpers.
//!

use dbuf_core::ast::elaborated::{Constructor, ConstructorNames};

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
        } => true,
        Symbol::Constructor {
            type_name: _,
            constructor: _,
        } => true,
        Symbol::None => false,
    }
}

trait BoolIfTrue {
    fn if_true(self, err: RenameError) -> Result<(), RenameError>;
}

impl BoolIfTrue for bool {
    fn if_true(self, err: RenameError) -> Result<(), RenameError> {
        (!self).then_some(()).ok_or(err)
    }
}

/// Check if symbol can be renamed to new_name without conflicts.
pub fn renameable_to_symbol(
    symbol: &Symbol,
    new_name: &str,
    ast: &ElaboratedAst,
) -> Result<(), RenameError> {
    new_name.is_empty().if_true(RenameError::ToEmpty)?;

    dbuf_language::get_builtin_types()
        .contains(new_name)
        .if_true(RenameError::ToBuiltin)?;

    dbuf_language::get_keywords()
        .contains(new_name)
        .if_true(RenameError::ToKeyword)?;

    match symbol {
        Symbol::Type { type_name } => {
            (type_name == new_name).if_true(RenameError::ToPrevious)?;

            dbuf_language::get_builtin_types()
                .contains(type_name)
                .if_true(RenameError::OfBuiltin)?;

            let new_type_name = new_name
                .try_into()
                .map_err(|_| RenameError::ToBadType(new_name.to_owned()))?;

            let checker = ConflictChecker::new(ast);

            checker
                .has_type_or_constructor(new_type_name)
                .if_true(RenameError::ToExistingType(new_name.to_owned()))?;
            Ok(())
        }
        Symbol::Dependency {
            type_name,
            dependency,
        } => {
            (dependency == new_name).if_true(RenameError::ToPrevious)?;

            let new_dependency_name = new_name
                .try_into()
                .map_err(|_| RenameError::ToBadDependency(new_name.to_owned()))?;

            let checker = ConflictChecker::new(ast);

            checker
                .type_has_resourse(type_name, new_dependency_name)
                .if_true(RenameError::ToExistingResource {
                    t: type_name.to_owned(),
                    r: new_name.to_owned(),
                })?;
            Ok(())
        }
        Symbol::Field {
            type_name,
            constructor: _,
            field,
        } => {
            (field == new_name).if_true(RenameError::ToPrevious)?;

            let new_field_name = new_name
                .try_into()
                .map_err(|_| RenameError::ToBadField(new_name.to_owned()))?;

            let checker = ConflictChecker::new(ast);

            checker
                .type_has_resourse(type_name, new_field_name)
                .if_true(RenameError::ToExistingResource {
                    t: type_name.to_owned(),
                    r: new_name.to_owned(),
                })?;
            Ok(())
        }
        Symbol::Alias {
            type_name,
            branch_id: _,
            alias,
        } => {
            (alias == new_name).if_true(RenameError::ToPrevious)?;

            let new_alias_name = new_name
                .try_into()
                .map_err(|_| RenameError::ToBadAlias(new_name.to_owned()))?;

            let checker = ConflictChecker::new(ast);

            checker
                .type_has_resourse(type_name, new_alias_name)
                .if_true(RenameError::ToExistingResource {
                    t: type_name.to_owned(),
                    r: new_name.to_owned(),
                })?;

            Ok(())
        }
        Symbol::Constructor {
            type_name: _,
            constructor,
        } => {
            (constructor == new_name).if_true(RenameError::ToPrevious)?;

            let new_constructor_name = new_name
                .try_into()
                .map_err(|_| RenameError::ToBadConstructor(new_name.to_owned()))?;

            let checker = ConflictChecker::new(ast);
            checker
                .has_type_or_constructor(new_constructor_name)
                .if_true(RenameError::ToExistingType(new_name.to_owned()))?;

            Ok(())
        }
        Symbol::None => Err(RenameError::OfNone),
    }
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
