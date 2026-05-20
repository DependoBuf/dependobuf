//! Module provides scope visitor, wich helps with calculating
//! scopes in parsed ast.
//!

use crate::core::workspace::ElaboratedAst;

mod ast_constructors_stack;
mod ast_scope;

use ast_constructors_stack::AstConstructorsStack;
use ast_scope::AstScope;

use super::*;

/// Scope visitior controls scopes.
///
/// Usage:
/// * Pass Visit token to him after your visit.
///
/// It is never returns anything but `VisitResult::Continue`,
/// so no checks need.
///
/// 'a is lifetime of parsed ast reference.
pub struct ScopeVisitor<'a> {
    /// branch id in enums or -1 in messages.
    branch_id: i32,
    /// current scope (type, constructor).
    scope: AstScope<'a>,
    /// constructors call stack.
    cons_stack: AstConstructorsStack<'a>,
}

impl<'a> ScopeVisitor<'a> {
    pub fn new(elaborated: &'a ElaboratedAst) -> ScopeVisitor<'a> {
        ScopeVisitor {
            branch_id: -1,
            scope: AstScope::new(elaborated),
            cons_stack: AstConstructorsStack::new(),
        }
    }

    /// Returns type of current scope.
    ///
    /// Returns None if not set, or if cannot
    /// be deduced in call chain.
    pub fn get_type(&self) -> Option<&'a str> {
        self.scope.get_type()
    }

    /// Returns constructor of current scope.
    ///
    /// Returns None if not set, or if cannot
    /// be deduced in call chain.
    pub fn get_constructor(&self) -> Option<&'a str> {
        self.scope.get_constructor()
    }

    /// Returns last constructor in constructors calls.
    ///
    /// Returns None if there is no constructors calls.
    pub fn get_constructor_expr(&self) -> Option<&'a str> {
        self.cons_stack.get_last()
    }

    /// Returns current `branch_id`.
    pub fn get_branch_id(&self) -> Option<usize> {
        if self.branch_id >= 0 && self.branch_id <= 1_000_000_000 {
            usize::try_from(self.branch_id).unwrap().into()
        } else {
            None
        }
    }
}

impl<'a> Visitor<'a> for ScopeVisitor<'a> {
    type StopResult = ();

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        match visit {
            Visit::Keyword(_, _) => {}
            Visit::Type(type_name, _) => {
                self.branch_id = -1;
                self.scope.enter_in_type(type_name.as_ref());
            }
            Visit::Dependency(_, _) => {}
            Visit::Branch => {
                assert!(self.cons_stack.is_empty());
                self.branch_id += 1;
            }
            Visit::PatternAlias(_) => {}
            Visit::PatternCall(cons, _) => self.cons_stack.enter_constructor(cons.as_ref()),
            Visit::PatternCallArgument(_) => {}
            Visit::PatternCallStop => self.cons_stack.leave_constructor(),
            Visit::PatternLiteral(_, _) => {}
            Visit::PatternUnderscore(_) => {}
            Visit::Constructor(cons) => self.scope.enter_in_constructor(cons.name.as_ref()),
            Visit::Filed(_, _) => {}
            Visit::TypeExpression(_, _) => {}
            Visit::Expression(_) => {}
            Visit::AccessChainStart => self.scope.save_state(),
            Visit::AccessChain(access) => self.scope.apply_variable(access.as_ref()),
            Visit::AccessDot(_) => {}
            Visit::AccessChainLast(_) => self.scope.load_state(),
            Visit::ConstructorExpr(cons) => self.cons_stack.enter_constructor(cons.as_ref()),
            Visit::ConstructorExprArgument(_) => {}
            Visit::ConstructorExprStop => self.cons_stack.leave_constructor(),
            Visit::VarAccess(_) => {}
            Visit::Operator(_, _) => {}
            Visit::Literal(_, _) => {}
        }

        VisitResult::Continue
    }
}
