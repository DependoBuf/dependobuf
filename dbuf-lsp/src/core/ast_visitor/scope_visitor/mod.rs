//! Module provides scope visitor, wich helps with calculating
//! scopes in parsed ast.
//!

use crate::core::ast_access::ElaboratedAst;

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
    pub branch_id: i32,
    /// current scope (type, constructor).
    pub scope: AstScope<'a>,
    /// constructors call stack.
    pub cons_stack: AstConstructorsStack<'a>,
}

impl<'a> ScopeVisitor<'a> {
    pub fn new(elaborated: &'a ElaboratedAst) -> ScopeVisitor<'a> {
        ScopeVisitor {
            branch_id: -1,
            scope: AstScope::new(elaborated),
            cons_stack: AstConstructorsStack::new(),
        }
    }

    /// Returns if type is set
    pub fn has_type(&self) -> bool {
        self.scope.has_type()
    }

    /// Checks if type is set and if so, returns it.
    ///
    /// Panics if type is not set.
    pub fn get_type(&self) -> &'a str {
        self.scope.get_type()
    }

    /// Returns if constructor is set
    pub fn has_constructor(&self) -> bool {
        self.scope.has_constructor()
    }

    /// Checks if constructor is set and if so, returns it.
    ///
    /// Panics if constructor is not set.
    pub fn get_constructor(&self) -> &'a str {
        self.scope.get_constructor()
    }

    /// Checks if constructors calls is not empty.
    pub fn has_constructor_expr(&self) -> bool {
        !self.cons_stack.is_empty()
    }
    /// Returns last constructor in constructors calls.
    ///
    /// Panics if there is no constructor calls.
    pub fn get_constructor_expr(&self) -> &'a str {
        self.cons_stack.get_last()
    }

    /// Returns if `branch_id` is set
    pub fn has_branch_id(&self) -> bool {
        self.branch_id >= 0 && self.branch_id <= 1_000_000_000
    }

    /// Returns current `branch_id`.
    ///
    /// Panics if it is not set.
    pub fn get_branch_id(&self) -> usize {
        assert!(self.has_branch_id());
        usize::try_from(self.branch_id).unwrap()
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
