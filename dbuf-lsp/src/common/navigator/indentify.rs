//! Indetifies symbol at location and returns it type
//! TODO:
//! * think about inconstructuble types (probably, no need)
//! * parse enums
//! * parse constructors in future
//!
use dbuf_core::ast::operators::{OpCall, UnaryOp};
use tower_lsp::lsp_types::Position;

use dbuf_core::ast::{elaborated, parsed::*};

use crate::common::ast_access::{ElaboratedHelper, LocStringHelper, LocationHelpers};
use crate::common::ast_access::{Loc, Str};

use super::Navigator;
use super::Symbol;

struct GetImpl<'a> {
    navigator: &'a Navigator<'a>,
    target: Position,
    t: String,
    constructor: String,
}

pub fn get_symbol_impl(navigator: &Navigator, pos: Position) -> Symbol {
    let mut implementation = GetImpl {
        navigator,
        target: pos,
        t: String::new(),
        constructor: String::new(),
    };

    implementation.get()
}

impl GetImpl<'_> {
    fn is_dependency(&self, dep: &Str) -> bool {
        if let Some(t) = self.navigator.elaborated.get_type(&self.t) {
            for (d, _) in t.dependencies.iter() {
                if d != dep.as_ref() {
                    continue;
                }
                return true;
            }
            return false;
        }
        panic!("no type {:?} in elaborated ast", dep);
    }
    fn is_field(&self, field: &Str) -> bool {
        let ctr_name = &self.constructor;
        let ctr = self.navigator.elaborated.constructors.get(ctr_name);
        if let Some(ctr) = ctr {
            for (f, _) in ctr.fields.iter() {
                if f != field.as_ref() {
                    continue;
                }
                return true;
            }
            return false;
        }
        panic!("constructor {:?} not found in elaborated ast", field);
    }

    fn setup_constructor_if_need(&mut self) {
        if self.constructor.is_empty() {
            let ast = self.navigator.elaborated;
            if ast.is_message(&self.t) {
                self.constructor = self.t.clone();
            }
        }
    }
    fn apply_variable(&mut self, var: &Str) {
        self.setup_constructor_if_need();

        let ctr_name = &self.constructor;
        let ctr = self.navigator.elaborated.constructors.get(ctr_name);

        if let Some(ctr) = ctr {
            for (i, te) in ctr.implicits.iter() {
                if i != var.as_ref() {
                    continue;
                }

                if let elaborated::Expression::Type {
                    name,
                    dependencies: _,
                } = te
                {
                    self.constructor = "".to_owned();
                    self.t = name.to_owned();
                    self.setup_constructor_if_need();
                    return;
                }

                panic!("bad type expression");
            }
            for (f, te) in ctr.fields.iter() {
                if f != var.as_ref() {
                    continue;
                }

                if let elaborated::Expression::Type {
                    name,
                    dependencies: _,
                } = te
                {
                    self.constructor = "".to_owned();
                    self.t = name.to_owned();
                    self.setup_constructor_if_need();
                    return;
                }

                panic!("bad type expression");
            }
        }
        panic!("constructor {:?} not found in elaborated ast", ctr_name);
    }

    fn apply_access_chain(&mut self, op: &Expression<Loc, Str>) {
        match &op.node {
            ExpressionNode::FunCall { fun, args } => {
                if !args.is_empty() {
                    panic!("bad access chain");
                }
                self.apply_variable(fun);
            }
            ExpressionNode::OpCall(op) => {
                if let OpCall::Unary(UnaryOp::Access(s), rhs) = op {
                    self.apply_access_chain(rhs);
                    self.apply_variable(s);
                    return;
                }
                panic!("bad access chain");
            }
            _ => {
                panic!("bad access chain");
            }
        }
    }

    fn get(&mut self) -> Symbol {
        for definition in self.navigator.parsed.iter() {
            if !definition.loc.contains(&self.target) {
                continue;
            }
            if definition.name.get_location().contains(&self.target) {
                return Symbol::Type(definition.name.to_string());
            }
            self.t = definition.name.to_string();
            return self.get_in_type(definition);
        }
        Symbol::None
    }

    fn get_in_type(&mut self, t: &TypeDeclaration<Loc, Str>) -> Symbol {
        for dependency in t.dependencies.iter() {
            if !dependency.loc.contains(&self.target) {
                continue;
            }
            if dependency.name.get_location().contains(&self.target) {
                return Symbol::Dependency {
                    t: std::mem::take(&mut self.t),
                    dependency: dependency.name.to_string(),
                };
            }
            return self.get_in_type_expr(dependency);
        }
        if let TypeDefinition::Message(body) = &t.body {
            self.constructor = self.t.clone();
            return self.get_in_constructor(body);
        }

        if let TypeDefinition::Enum(_branches) = &t.body {
            panic!("indentify symbol in enums not implemented");
        }

        Symbol::None
    }

    fn get_in_constructor(&mut self, ctr: &ConstructorBody<Loc, Str>) -> Symbol {
        for field in ctr.iter() {
            if !field.loc.contains(&self.target) {
                continue;
            }
            if field.name.get_location().contains(&self.target) {
                return Symbol::Field {
                    t: std::mem::take(&mut self.t),
                    constructor: std::mem::take(&mut self.constructor),
                    field: field.name.to_string(),
                };
            }
            return self.get_in_type_expr(field);
        }

        Symbol::None
    }

    fn get_in_type_expr(&mut self, te: &TypeExpression<Loc, Str>) -> Symbol {
        if !te.loc.contains(&self.target) {
            return Symbol::None;
        }

        if let ExpressionNode::FunCall { fun, args } = &te.node {
            if fun.get_location().contains(&self.target) {
                return Symbol::Type(fun.to_string());
            }
            for expr in args.iter() {
                if !expr.loc.contains(&self.target) {
                    continue;
                }
                return self.get_in_expr(expr);
            }

            return Symbol::None;
        }

        panic!("bad type expression");
    }

    fn get_in_expr(&mut self, expr: &Expression<Loc, Str>) -> Symbol {
        if !expr.loc.contains(&self.target) {
            return Symbol::None;
        }

        match &expr.node {
            ExpressionNode::FunCall { fun, args } => {
                if !args.is_empty() {
                    // TODO: parse constructor
                    panic!("constructors API will be changed");
                }
                self.get_variable(fun)
            }
            ExpressionNode::OpCall(op) => match op {
                OpCall::Literal(_) => Symbol::None,
                OpCall::Binary(_, lhs, rhs) => {
                    if lhs.loc.contains(&self.target) {
                        return self.get_in_expr(lhs);
                    }
                    if rhs.loc.contains(&self.target) {
                        return self.get_in_expr(rhs);
                    }
                    Symbol::None
                }
                OpCall::Unary(op, rhs) => {
                    if let UnaryOp::Access(s) = op {
                        if s.get_location().contains(&self.target) {
                            self.apply_access_chain(rhs);
                            return self.get_variable(s);
                        }
                    }
                    self.get_in_expr(rhs)
                }
            },
            _ => {
                panic!("bad expression node")
            }
        }
    }

    fn get_variable(&mut self, variable: &Str) -> Symbol {
        if !variable.get_location().contains(&self.target) {
            return Symbol::None;
        }
        // Variable should be either dependency or field
        // In future, it might be part of constructors chain
        if self.is_dependency(variable) {
            return Symbol::Dependency {
                t: std::mem::take(&mut self.t),
                dependency: variable.to_string(),
            };
        }
        if self.is_field(variable) {
            return Symbol::Field {
                t: std::mem::take(&mut self.t),
                constructor: std::mem::take(&mut self.constructor),
                field: variable.to_string(),
            };
        }
        panic!("bad variable expr")
    }
}
