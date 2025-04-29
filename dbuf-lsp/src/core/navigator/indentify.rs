//! Indetifies symbol at location and returns it type.
//!

use tower_lsp::lsp_types::Position;

use crate::core::ast_access::{
    ElaboratedAst, ElaboratedHelper, LocStringHelper, LocationHelpers, Str,
};

use crate::core::ast_visitor::scope_visitor::ScopeVisitor;
use crate::core::ast_visitor::VisitResult::*;
use crate::core::ast_visitor::*;

use super::Navigator;
use super::Symbol;

struct GetImpl<'a> {
    elaborated: &'a ElaboratedAst,
    target: Position,
    scope: ScopeVisitor<'a>,
    result: Symbol,
}

pub fn get_symbol_impl(navigator: &Navigator, pos: Position) -> Symbol {
    let mut implementation = GetImpl {
        elaborated: navigator.elaborated,
        target: pos,
        scope: ScopeVisitor::new(navigator.elaborated),
        result: Symbol::None,
    };

    visit_ast(navigator.parsed, &mut implementation, navigator.elaborated);

    implementation.result
}

impl GetImpl<'_> {
    fn no_result(&self) -> bool {
        matches!(self.result, Symbol::None)
    }

    fn return_type(&mut self, type_name: &Str) {
        assert!(self.no_result());
        self.result = Symbol::Type(type_name.to_string());
    }

    fn return_dependency(&mut self, dependency: &Str) {
        assert!(self.no_result());
        self.result = Symbol::Dependency {
            t: self.scope.get_type().to_owned(),
            dependency: dependency.to_string(),
        };
    }

    fn return_field(&mut self, field: &Str) {
        assert!(self.no_result());
        self.result = Symbol::Field {
            constructor: self.scope.get_constructor().to_owned(),
            field: field.to_string(),
        };
    }

    fn return_alias(&mut self, alias: &Str) {
        assert!(self.no_result());
        self.result = Symbol::Alias {
            t: self.scope.get_type().to_owned(),
            branch_id: self.scope.get_branch_id(),
            name: alias.to_string(),
        };
    }

    fn return_constructor(&mut self, constructor: &Str) {
        assert!(self.no_result());

        if self.elaborated.is_message(constructor.as_ref()) {
            self.result = Symbol::Type(constructor.to_string());
        } else {
            self.result = Symbol::Constructor(constructor.to_string());
        }
    }

    fn return_access(&mut self, access: &Str) {
        assert!(access.get_location().contains(&self.target));

        // Variable should be either dependency or field
        if self
            .elaborated
            .is_type_dependency(self.scope.get_type(), access.as_ref())
        {
            self.return_dependency(access);
        } else if self
            .elaborated
            .is_constructor_field(self.scope.get_constructor(), access.as_ref())
        {
            self.return_field(access);
        } else {
            panic!("bad variable expr")
        }
    }
}

impl<'a> Visitor<'a> for GetImpl<'a> {
    fn visit(&mut self, visit: Visit<'a>) -> VisitResult {
        match &visit {
            Visit::Keyword(_, _) => {}
            Visit::Type(type_name, type_location) => {
                if !type_location.contains(&self.target) {
                    return Skip;
                }
                if type_name.get_location().contains(&self.target) {
                    self.return_type(type_name);
                    return Stop;
                }
            }
            Visit::Dependency(dep_name, dependency_location) => {
                if !dependency_location.contains(&self.target) {
                    return Skip;
                }
                if dep_name.get_location().contains(&self.target) {
                    self.return_dependency(dep_name);
                    return Stop;
                }
            }
            Visit::Branch => {}
            Visit::PatternAlias(alias) => {
                if alias.get_location().contains(&self.target) {
                    self.return_alias(alias);
                    return Stop;
                }
            }
            Visit::PatternCall(constructor, loc) => {
                if !loc.contains(&self.target) {
                    return Skip;
                }
                if constructor.get_location().contains(&self.target) {
                    self.return_constructor(constructor);
                    return Stop;
                }
            }
            Visit::PatternCallArgument(_loc_string) => {
                panic!("constructor call argument name is not implemented");
            }
            Visit::PatternCallStop => {}
            Visit::PatternLiteral(_, _) => {}
            Visit::PatternUnderscore(_) => {}
            Visit::Constructor(constructor) => {
                if !constructor.loc.contains(&self.target) {
                    return Skip;
                }
                if constructor.name.get_location().contains(&self.target) {
                    self.return_constructor(constructor.name);
                    return Stop;
                }
            }
            Visit::Filed(field, loc) => {
                if !loc.contains(&self.target) {
                    return Skip;
                }
                if field.get_location().contains(&self.target) {
                    self.return_field(field);
                    return Stop;
                }
            }
            Visit::TypeExpression(type_name, loc) => {
                if !loc.contains(&self.target) {
                    return Skip;
                }
                if type_name.get_location().contains(&self.target) {
                    self.return_type(type_name);
                    return Stop;
                }
            }
            Visit::Expression(loc) => {
                if !loc.contains(&self.target) {
                    return Skip;
                }
            }
            Visit::AccessChainStart => {}
            Visit::AccessChain(access) => {
                if access.get_location().contains(&self.target) {
                    self.return_access(access);
                    return Stop;
                }
            }
            Visit::AccessDot(_) => {}
            Visit::AccessChainLast(access) => {
                if access.get_location().contains(&self.target) {
                    self.return_access(access);
                    return Stop;
                }
            }
            Visit::ConstructorExpr(constructor) => {
                if constructor.get_location().contains(&self.target) {
                    self.return_constructor(constructor);
                    return Stop;
                }
            }
            Visit::ConstructorExprArgument(_loc_string) => {
                panic!("constructor call argument name is not implemented");
            }
            Visit::ConstructorExprStop => {}
            Visit::VarAccess(access) => {
                if access.get_location().contains(&self.target) {
                    self.return_access(access);
                    return Stop;
                }
            }
            Visit::Operator(_, _) => {}
            Visit::Literal(_, _) => {}
        };

        assert!(matches!(self.scope.visit(visit), VisitResult::Continue));

        Continue
    }
}
