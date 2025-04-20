use crate::common::ast_access::ElaboratedAst;
use crate::common::ast_constructors_stack::AstConstructorsStack;
use crate::common::ast_scope::AstScope;

use super::*;

pub struct ScopeVisitor<'a> {
    pub branch_id: i32,
    pub scope: AstScope<'a>,
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

    pub fn get_type(&self) -> &'a str {
        self.scope.get_type()
    }

    pub fn get_option_type(&self) -> &'a str {
        self.scope.get_option_type()
    }

    pub fn get_constructor(&self) -> &'a str {
        self.scope.get_constructor()
    }

    pub fn get_option_constructor(&self) -> &'a str {
        self.scope.get_option_constructor()
    }

    pub fn get_constructor_expr(&self) -> &'a str {
        self.cons_stack.get_last()
    }

    pub fn get_branch_id(&self) -> usize {
        assert!(self.branch_id >= 0);
        assert!(self.branch_id <= 1e9 as i32);
        self.branch_id as usize
    }
}

impl<'a> Visitor<'a> for ScopeVisitor<'a> {
    fn visit(&mut self, visit: Visit<'a>) -> VisitResult {
        match visit {
            Visit::Keyword(_, _) => {}
            Visit::Type(type_name, _) => {
                self.branch_id = -1;
                self.scope.enter_into_type(type_name.as_ref());
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
            Visit::Constructor(cons) => self.scope.enter_into_constructor(cons.name.as_ref()),
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
