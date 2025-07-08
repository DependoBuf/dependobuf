//! find all locations of symbol
//! TODO:
//! * maskings issues in patterns
//!

use tower_lsp::lsp_types::Range;

use crate::core::ast_access::{LocStringHelper, LocationHelpers, Str};

use crate::core::ast_visitor::scope_visitor::ScopeVisitor;
use crate::core::ast_visitor::*;

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
    fn correct_symbol(&self, str: &Str) -> bool {
        match &self.target {
            Symbol::Type { type_name } => type_name == str.as_ref(),
            Symbol::Dependency {
                type_name,
                dependency,
            } => {
                self.scope.has_type()
                    && type_name == self.scope.get_type()
                    && dependency == str.as_ref()
            }
            Symbol::Field {
                type_name,
                constructor,
                field,
            } => {
                self.scope.has_type()
                    && type_name == self.scope.get_type()
                    && self.scope.has_constructor()
                    && constructor == self.scope.get_constructor()
                    && field == str.as_ref()
            }
            Symbol::Alias {
                type_name,
                branch_id,
                alias,
            } => {
                self.scope.has_type()
                    && type_name == self.scope.get_type()
                    && self.scope.has_branch_id()
                    && *branch_id == self.scope.get_branch_id()
                    && alias == str.as_ref()
            }
            Symbol::Constructor {
                type_name: _,
                constructor,
            } => constructor == str.as_ref(),
            Symbol::None => false,
        }
    }

    fn correct_argument_symbol(&self, str: &Str) -> bool {
        match &self.target {
            Symbol::Field {
                type_name: _,
                constructor,
                field,
            } => {
                self.scope.has_constructor_expr()
                    && self.scope.get_constructor_expr() == constructor
                    && field == str.as_ref()
            }
            _ => false,
        }
    }

    fn push(&mut self, str: &Str) {
        self.ans.push(str.get_location().to_lsp());
    }
}

impl<'a> Visitor<'a> for FindImpl<'a> {
    type StopResult = ();

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match &visit {
            Visit::Type(str, _) if self.correct_symbol(str) => self.push(str),
            Visit::Dependency(str, _) if self.correct_symbol(str) => self.push(str),
            Visit::PatternAlias(str) if self.correct_symbol(str) => self.push(str),
            Visit::PatternCall(str, _) if self.correct_symbol(str) => self.push(str),
            Visit::PatternCallArgument(str) if self.correct_argument_symbol(str) => self.push(str),
            Visit::Constructor(cons) if !cons.of_message && self.correct_symbol(cons.name) => {
                self.push(cons.name);
            }
            Visit::Filed(str, _) if self.correct_symbol(str) => self.push(str),
            Visit::TypeExpression(str, _) if self.correct_symbol(str) => self.push(str),
            Visit::AccessChain(str) if self.correct_symbol(str) => self.push(str),
            Visit::AccessChainLast(str) if self.correct_symbol(str) => self.push(str),
            Visit::ConstructorExpr(str) if self.correct_symbol(str) => self.push(str),
            Visit::ConstructorExprArgument(str) if self.correct_argument_symbol(str) => {
                self.push(str);
            }
            Visit::VarAccess(str) if self.correct_symbol(str) => self.push(str),
            _ => {}
        }

        assert!(matches!(self.scope.visit(visit), VisitResult::Continue));

        VisitResult::Continue
    }
}
