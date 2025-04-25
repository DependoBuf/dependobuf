//! find all locations of symbol
//! TODO:
//! * maskings issues in patterns
//!

use tower_lsp::lsp_types::Range;

use crate::common::ast_access::{LocStringHelper, LocationHelpers, Str};

use crate::common::ast_visitor::scope_visitor::ScopeVisitor;
use crate::common::ast_visitor::*;

use super::Navigator;
use super::Symbol;

struct FindImpl<'a> {
    target: &'a Symbol,
    scope: ScopeVisitor<'a>,
    ans: Vec<Range>,
}

pub fn find_symbols_impl(navigator: &Navigator, symbol: &Symbol) -> Vec<Range> {
    let mut implementation = FindImpl {
        target: symbol,
        scope: ScopeVisitor::new(navigator.elaborated),
        ans: Vec::new(),
    };

    visit_ast(navigator.parsed, &mut implementation, navigator.elaborated);

    implementation.ans
}

impl FindImpl<'_> {
    fn check_add(&mut self, str: &Str) {
        match &self.target {
            Symbol::Type(type_name) => {
                if type_name != str.as_ref() {
                    return;
                }
            }
            Symbol::Dependency { t, dependency } => {
                if !self.scope.has_type() {
                    return;
                }
                if t != self.scope.get_type() {
                    return;
                }
                if dependency != str.as_ref() {
                    return;
                }
            }
            Symbol::Field { constructor, field } => {
                if !self.scope.has_constructor() {
                    return;
                }
                if constructor != self.scope.get_constructor() {
                    return;
                }
                if field != str.as_ref() {
                    return;
                }
            }
            Symbol::Alias { t, branch_id, name } => {
                if !self.scope.has_type() {
                    return;
                }
                if t != self.scope.get_type() {
                    return;
                }
                if *branch_id != self.scope.get_branch_id() {
                    return;
                }
                if name != str.as_ref() {
                    return;
                }
            }
            Symbol::Constructor(constructor) => {
                if constructor != str.as_ref() {
                    return;
                }
            }
            Symbol::None => return,
        }
        self.ans.push(str.get_location().to_lsp());
    }
}

impl<'a> Visitor<'a> for FindImpl<'a> {
    fn visit(&mut self, visit: Visit<'a>) -> VisitResult {
        match &visit {
            Visit::Keyword(_, _) => {}
            Visit::Type(type_name, _) => self.check_add(type_name),
            Visit::Dependency(dep_name, _) => self.check_add(dep_name),
            Visit::Branch => {}
            Visit::PatternAlias(alias) => self.check_add(alias),
            Visit::PatternCall(cons, _) => self.check_add(cons),
            Visit::PatternCallArgument(name) => {
                assert!(name.is_none(), "naming not supported");
            }
            Visit::PatternCallStop => {}
            Visit::PatternLiteral(_, _) => {}
            Visit::PatternUnderscore(_) => {}
            Visit::Constructor(cons) => {
                if !cons.of_message {
                    self.check_add(cons.name)
                }
            }
            Visit::Filed(field, _) => self.check_add(field),
            Visit::TypeExpression(type_name, _) => self.check_add(type_name),
            Visit::Expression(_) => {}
            Visit::AccessChainStart => {}
            Visit::AccessChain(access) => self.check_add(access),
            Visit::AccessDot(_) => {}
            Visit::AccessChainLast(access) => self.check_add(access),
            Visit::ConstructorExpr(cons) => self.check_add(cons),
            Visit::ConstructorExprArgument(name) => {
                assert!(name.is_none(), "naming not supported");
            }
            Visit::ConstructorExprStop => {}
            Visit::VarAccess(access) => self.check_add(access),
            Visit::Operator(_, _) => {}
            Visit::Literal(_, _) => {}
        }

        assert!(matches!(self.scope.visit(visit), VisitResult::Continue));

        VisitResult::Continue
    }
}
