//! Indetifies symbol at location and returns it type.
//!

use tower_lsp::lsp_types::Position;

use crate::core::ast_access::{
    ElaboratedAst, ElaboratedHelper, LocStringHelper, LocationHelpers, Str,
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

    let res = visit_ast(navigator.parsed, &mut implementation, navigator.elaborated);

    res.unwrap_or(Symbol::None)
}

impl GetImpl<'_> {
    fn get_type(&self, type_name: &Str) -> Symbol {
        assert!(type_name.contains(self.target));

        Symbol::Type {
            type_name: type_name.to_string(),
        }
    }

    fn get_dependency(&self, dependency: &Str) -> Symbol {
        assert!(dependency.contains(self.target));

        Symbol::Dependency {
            type_name: self.scope.get_type().to_owned(),
            dependency: dependency.to_string(),
        }
    }

    fn get_field(&self, field: &Str) -> Symbol {
        assert!(field.contains(self.target));

        Symbol::Field {
            type_name: self.scope.get_type().to_owned(),
            constructor: self.scope.get_constructor().to_owned(),
            field: field.to_string(),
        }
    }

    fn get_alias(&self, alias: &Str) -> Symbol {
        assert!(alias.contains(self.target));

        Symbol::Alias {
            type_name: self.scope.get_type().to_owned(),
            branch_id: self.scope.get_branch_id(),
            alias: alias.to_string(),
        }
    }

    fn get_argument(&self, argument: &Str) -> Symbol {
        let constructor = self.scope.get_constructor_expr();
        let type_name = self
            .elaborated
            .get_constructor_type(constructor)
            .expect("correctly builded AST");

        Symbol::Field {
            type_name: type_name.to_owned(),
            constructor: constructor.to_owned(),
            field: argument.to_string(),
        }
    }

    fn get_constructor(&self, constructor: &Str) -> Symbol {
        assert!(constructor.contains(self.target));

        if self.elaborated.is_message(constructor.as_ref()) {
            Symbol::Type {
                type_name: constructor.to_string(),
            }
        } else {
            Symbol::Constructor {
                type_name: self.scope.get_type().to_owned(),
                constructor: constructor.to_string(),
            }
        }
    }

    fn get_access(&self, access: &Str) -> Symbol {
        assert!(access.contains(self.target));

        // Variable should be one of: dependency, field, alias
        if self
            .elaborated
            .is_type_dependency(self.scope.get_type(), access.as_ref())
        {
            self.get_dependency(access)
        } else if self
            .elaborated
            .is_constructor_field(self.scope.get_constructor(), access.as_ref())
        {
            self.get_field(access)
        } else if self
            .elaborated
            .is_constructor_implicit(self.scope.get_constructor(), access.as_ref())
        {
            self.get_alias(access)
        } else {
            panic!("bad variable expr")
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
                Stop(self.get_constructor(constructor))
            }
            Visit::PatternCallArgument(argument) if argument.contains(self.target) => {
                Stop(self.get_argument(argument))
            }
            Visit::Constructor(constructor) if !constructor.loc.contains(self.target) => Skip,
            Visit::Constructor(constructor) if constructor.name.contains(self.target) => {
                Stop(self.get_constructor(constructor.name))
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
                Stop(self.get_constructor(constructor))
            }
            Visit::ConstructorExprArgument(argument) if argument.contains(self.target) => {
                Stop(self.get_argument(argument))
            }
            Visit::VarAccess(access) if access.contains(self.target) => {
                Stop(self.get_access(access))
            }
            _ => {
                assert!(matches!(self.scope.visit(visit), VisitResult::Continue));
                Continue
            }
        }
    }
}
