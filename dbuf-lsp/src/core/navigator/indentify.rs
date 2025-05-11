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
        assert!(type_name.get_location().contains(self.target));

        Symbol::Type {
            type_name: type_name.to_string(),
        }
    }

    fn get_dependency(&self, dependency: &Str) -> Symbol {
        assert!(dependency.get_location().contains(self.target));

        Symbol::Dependency {
            type_name: self.scope.get_type().to_owned(),
            dependency: dependency.to_string(),
        }
    }

    fn get_field(&self, field: &Str) -> Symbol {
        assert!(field.get_location().contains(self.target));

        Symbol::Field {
            type_name: self.scope.get_type().to_owned(),
            constructor: self.scope.get_constructor().to_owned(),
            field: field.to_string(),
        }
    }

    fn get_alias(&self, alias: &Str) -> Symbol {
        assert!(alias.get_location().contains(self.target));

        Symbol::Alias {
            type_name: self.scope.get_type().to_owned(),
            branch_id: self.scope.get_branch_id(),
            alias: alias.to_string(),
        }
    }

    fn get_constructor(&self, constructor: &Str) -> Symbol {
        assert!(constructor.get_location().contains(self.target));

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
        assert!(access.get_location().contains(self.target));

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
            Visit::Keyword(_, _) => {}
            Visit::Type(type_name, type_location) => {
                if !type_location.contains(self.target) {
                    return Skip;
                }
                if type_name.get_location().contains(self.target) {
                    return Stop(self.get_type(type_name));
                }
            }
            Visit::Dependency(dep_name, dependency_location) => {
                if !dependency_location.contains(self.target) {
                    return Skip;
                }
                if dep_name.get_location().contains(self.target) {
                    return Stop(self.get_dependency(dep_name));
                }
            }
            Visit::Branch => {}
            Visit::PatternAlias(alias) => {
                if alias.get_location().contains(self.target) {
                    return Stop(self.get_alias(alias));
                }
            }
            Visit::PatternCall(constructor, loc) => {
                if !loc.contains(self.target) {
                    return Skip;
                }
                if constructor.get_location().contains(self.target) {
                    return Stop(self.get_constructor(constructor));
                }
            }
            Visit::PatternCallArgument(_loc_string) => {
                panic!("constructor call argument name is not implemented");
            }
            Visit::PatternCallStop => {}
            Visit::PatternLiteral(_, _) => {}
            Visit::PatternUnderscore(_) => {}
            Visit::Constructor(constructor) => {
                if !constructor.loc.contains(self.target) {
                    return Skip;
                }
                if constructor.name.get_location().contains(self.target) {
                    return Stop(self.get_constructor(constructor.name));
                }
            }
            Visit::Filed(field, loc) => {
                if !loc.contains(self.target) {
                    return Skip;
                }
                if field.get_location().contains(self.target) {
                    return Stop(self.get_field(field));
                }
            }
            Visit::TypeExpression(type_name, loc) => {
                if !loc.contains(self.target) {
                    return Skip;
                }
                if type_name.get_location().contains(self.target) {
                    return Stop(self.get_type(type_name));
                }
            }
            Visit::Expression(loc) => {
                if !loc.contains(self.target) {
                    return Skip;
                }
            }
            Visit::AccessChainStart => {}
            Visit::AccessChain(access) => {
                if access.get_location().contains(self.target) {
                    return Stop(self.get_access(access));
                }
            }
            Visit::AccessDot(_) => {}
            Visit::AccessChainLast(access) => {
                if access.get_location().contains(self.target) {
                    return Stop(self.get_access(access));
                }
            }
            Visit::ConstructorExpr(constructor) => {
                if constructor.get_location().contains(self.target) {
                    return Stop(self.get_constructor(constructor));
                }
            }
            Visit::ConstructorExprArgument(_loc_string) => {
                panic!("constructor call argument name is not implemented");
            }
            Visit::ConstructorExprStop => {}
            Visit::VarAccess(access) => {
                if access.get_location().contains(self.target) {
                    return Stop(self.get_access(access));
                }
            }
            Visit::Operator(_, _) => {}
            Visit::Literal(_, _) => {}
        };

        assert!(matches!(self.scope.visit(visit), VisitResult::Continue));

        Continue
    }
}
