//! Indetifies symbol at location and returns it type.
//!

use dbuf_core::ast::elaborated::ConstructorNames;
use tower_lsp::lsp_types::Position;

use crate::core::workspace::{
    ElaboratedAst, ElaboratedHelper, LocNameHelper, LocationHelper, Name,
};

use crate::core::ast_visitor::VisitResult::*;
use crate::core::ast_visitor::scope_visitor::ScopeVisitor;
use crate::core::ast_visitor::*;

use super::Navigator;
use super::Symbol;

struct GetImpl<'a> {
    elaborated: &'a ElaboratedAst,
    target: Position,
    scope: ScopeVisitor<'a>,
}

pub fn get_symbol_impl(navigator: &Navigator, pos: Position) -> Symbol {
    let mut implementation = GetImpl {
        elaborated: navigator.elaborated,
        target: pos,
        scope: ScopeVisitor::new(navigator.elaborated),
    };

    let res = visit_ast(navigator.parsed, &mut implementation);

    res.unwrap_or(Symbol::None)
}

impl GetImpl<'_> {
    fn get_type(&self, type_name: &Name) -> Symbol {
        assert!(type_name.contains(self.target));

        Symbol::Type {
            type_name: type_name.to_string(),
        }
    }

    fn get_dependency(&self, dependency: &Name) -> Symbol {
        assert!(dependency.contains(self.target));

        let ty_name = self.scope.get_type().expect("since should be set");

        Symbol::Dependency {
            type_name: ty_name.to_owned(),
            dependency: dependency.to_string(),
        }
    }

    fn get_field(&self, field: &Name) -> Symbol {
        assert!(field.contains(self.target));

        let type_name = self
            .scope
            .get_type()
            .expect("since should be set")
            .to_owned();
        let constructor = self
            .scope
            .get_constructor()
            .expect("since should be set")
            .to_owned();

        Symbol::Field {
            type_name,
            constructor,
            field: field.to_string(),
        }
    }

    fn get_alias(&self, alias: &Name) -> Symbol {
        assert!(alias.contains(self.target));

        let type_name = self
            .scope
            .get_type()
            .expect("since should be set")
            .to_owned();
        let branch_id = self.scope.get_branch_id().expect("since should be set");

        Symbol::Alias {
            type_name,
            branch_id,
            alias: alias.to_string(),
        }
    }

    fn get_argument(&self, argument: &Name) -> Symbol {
        assert!(argument.contains(self.target));

        let constructor = self
            .scope
            .get_constructor_expr()
            .expect("since should be set");
        let Some(type_name) = self.elaborated.get_constructor_type(constructor) else {
            return Symbol::None;
        };

        Symbol::Field {
            type_name: type_name.to_owned(),
            constructor: constructor.to_owned(),
            field: argument.to_string(),
        }
    }

    fn get_constructor(&self, constructor: &Name, of_message: bool) -> Symbol {
        assert!(constructor.contains(self.target));

        if of_message {
            Symbol::Type {
                type_name: constructor.to_string(),
            }
        } else {
            let type_name = self
                .scope
                .get_type()
                .expect("since should be set")
                .to_owned();
            Symbol::Constructor {
                type_name,
                constructor: constructor.to_string(),
            }
        }
    }

    fn get_constructor_call(&self, constructor: &Name) -> Symbol {
        assert!(constructor.contains(self.target));

        let Some(type_name) = self.elaborated.get_constructor_type(constructor.as_ref()) else {
            return Symbol::None;
        };

        let Some(ty) = self.elaborated.get_type(type_name) else {
            return Symbol::None;
        };

        match ty.constructor_names {
            ConstructorNames::OfMessage(_) => Symbol::Type {
                type_name: constructor.to_string(),
            },
            ConstructorNames::OfEnum(_) => Symbol::Constructor {
                type_name: type_name.to_owned(),
                constructor: constructor.to_string(),
            },
        }
    }

    fn get_access(&self, access: &Name) -> Symbol {
        assert!(access.contains(self.target));

        // Variable should be one of: dependency, field, alias
        let Some(ty_name) = self.scope.get_type() else {
            return Symbol::None;
        };

        if self.elaborated.is_type_dependency(ty_name, access.as_ref()) {
            return self.get_dependency(access);
        }

        let Some(ctr_name) = self.scope.get_constructor() else {
            return Symbol::None;
        };

        if self
            .elaborated
            .is_constructor_field(ctr_name, access.as_ref())
        {
            return self.get_field(access);
        }

        if self
            .elaborated
            .is_constructor_implicit(ctr_name, access.as_ref())
        {
            self.get_alias(access)
        } else {
            // Not enough information in EAST to deduce type
            Symbol::None
        }
    }
}

impl<'a> Visitor<'a> for GetImpl<'a> {
    type StopResult = Symbol;

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match &visit {
            Visit::Type(_, loc) if !loc.contains(self.target) => Skip,
            Visit::Type(type_name, _) if type_name.contains(self.target) => {
                Stop(self.get_type(type_name))
            }
            Visit::Dependency(_, loc) if !loc.contains(self.target) => Skip,
            Visit::Dependency(dependency, _) if dependency.contains(self.target) => {
                Stop(self.get_dependency(dependency))
            }
            Visit::PatternAlias(alias) if alias.contains(self.target) => {
                Stop(self.get_alias(alias))
            }
            Visit::PatternCall(_, loc) if !loc.contains(self.target) => Skip,
            Visit::PatternCall(constructor, _) if constructor.contains(self.target) => {
                Stop(self.get_constructor_call(constructor))
            }
            Visit::PatternCallArgument(argument) if argument.contains(self.target) => {
                Stop(self.get_argument(argument))
            }
            Visit::Constructor(constructor) if !constructor.loc.contains(self.target) => Skip,
            Visit::Constructor(constructor) if constructor.name.contains(self.target) => {
                Stop(self.get_constructor(constructor.name, constructor.of_message))
            }
            Visit::Filed(_, loc) if !loc.contains(self.target) => Skip,
            Visit::Filed(field, _) if field.contains(self.target) => Stop(self.get_field(field)),
            Visit::TypeExpression(_, loc) if !loc.contains(self.target) => Skip,
            Visit::TypeExpression(type_name, _) if type_name.contains(self.target) => {
                Stop(self.get_type(type_name))
            }
            Visit::Expression(loc) if !loc.contains(self.target) => Skip,
            Visit::AccessChain(access) if access.contains(self.target) => {
                Stop(self.get_access(access))
            }
            Visit::AccessChainLast(access) if access.contains(self.target) => {
                Stop(self.get_access(access))
            }
            Visit::ConstructorExpr(constructor) if constructor.contains(self.target) => {
                Stop(self.get_constructor_call(constructor))
            }
            Visit::ConstructorExprArgument(argument) if argument.contains(self.target) => {
                Stop(self.get_argument(argument))
            }
            Visit::VarAccess(access) if access.contains(self.target) => {
                Stop(self.get_access(access))
            }
            _ => {
                let res = self.scope.visit(visit);

                assert!(matches!(res, VisitResult::Continue));
                Continue
            }
        }
    }
}
