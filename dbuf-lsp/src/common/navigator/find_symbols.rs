//! find all locations of symbol
//! TODO:
//! * modify in case of inconstructable type
//! * parse enums
//! * parse constructors in future
//!

use dbuf_core::ast::operators::{OpCall, UnaryOp};
use dbuf_core::ast::{elaborated, parsed::*};
use tower_lsp::lsp_types::Range;

use crate::common::ast_access::{Loc, Str};
use crate::common::ast_access::{LocStringHelper, LocationHelpers};

use super::Navigator;
use super::Symbol;

struct FindImpl<'a> {
    navigator: &'a Navigator<'a>,
    target: &'a Symbol,
    t: String,
    constructor: String,
    ans: Vec<Range>,
}

pub fn find_symbols_impl(navigator: &Navigator, symbol: &Symbol) -> Vec<Range> {
    let mut implementation = FindImpl {
        navigator,
        target: symbol,
        t: String::new(),
        constructor: String::new(),
        ans: Vec::new(),
    };

    implementation.find();

    implementation.ans
}

impl FindImpl<'_> {
    fn setup_constructor_if_need(&mut self) {
        if self.constructor.is_empty() {
            self.constructor = self.t.clone();
            /*
            eprintln!("type is {:?}", self.t);
            let t = &self.t;
            let ctr_name = self
                .navigator
                .elaborated
                .get_any_constructor(&t)
                .take()
                .expect("type is constructable");
            self.constructor = ctr_name.to_owned();
            */
        }
    }
    fn apply_variable(&mut self, var: &Str) {
        // TODO: modify in case of inconstructable type
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

    fn check_add(&mut self, str: &Str) {
        match self.target {
            Symbol::Type(t) => {
                if t != str.as_ref() {
                    return;
                }
                self.ans.push(str.get_location().to_lsp());
            }
            Symbol::Dependency { t, dependency } => {
                if t != &self.t {
                    return;
                }
                if dependency != str.as_ref() {
                    return;
                }
                self.ans.push(str.get_location().to_lsp());
            }
            Symbol::Field { constructor, field } => {
                if constructor != &self.constructor {
                    return;
                }
                if field != str.as_ref() {
                    return;
                }
                self.ans.push(str.get_location().to_lsp());
            }
            _ => {}
        }
    }

    fn find(&mut self) {
        if let Symbol::None = self.target {
            return;
        }
        if let Symbol::Constructor(_) = self.target {
            // TODO: not implemented
            return;
        }

        for definition in self.navigator.parsed.iter() {
            self.check_add(&definition.name);
            self.t = definition.name.to_string();
            self.find_in_type(definition);
        }
    }

    fn find_in_type(&mut self, t: &TypeDeclaration<Loc, Str>) {
        for dependency in t.dependencies.iter() {
            self.check_add(&dependency.name);
            self.find_in_type_expr(dependency);
        }
        match &t.body {
            TypeDefinition::Message(m) => {
                self.constructor = self.t.clone();
                self.find_in_constructor(m);
            }
            TypeDefinition::Enum(_e) => {
                panic!("find symbol in enum not implemented");
            }
        }
    }

    fn find_in_constructor(&mut self, ctr: &ConstructorBody<Loc, Str>) {
        for field in ctr.iter() {
            self.check_add(&field.name);
            self.find_in_type_expr(field);
        }
    }

    fn find_in_type_expr(&mut self, te: &TypeExpression<Loc, Str>) {
        match &te.node {
            ExpressionNode::FunCall { fun, args } => {
                self.check_add(fun);
                for expr in args.iter() {
                    self.find_in_expr(expr);
                }
            }
            _ => {
                panic!("bad type expression")
            }
        }
    }

    fn find_in_expr(&mut self, e: &Expression<Loc, Str>) {
        match &e.node {
            ExpressionNode::OpCall(op) => match op {
                OpCall::Binary(_, lhs, rhs) => {
                    self.find_in_expr(lhs);
                    self.find_in_expr(rhs);
                }
                OpCall::Unary(op, rhs) => {
                    if let UnaryOp::Access(_) = op {
                        self.find_in_access_chain(e);
                    } else {
                        self.find_in_expr(rhs);
                    }
                }
                OpCall::Literal(_) => {}
            },
            ExpressionNode::FunCall { fun, args } => {
                if !args.is_empty() {
                    panic!("construcors are not supported");
                }
                self.check_add(fun);
            }
            _ => {
                panic!("bad expression");
            }
        }
    }

    fn find_in_access_chain(&mut self, e: &Expression<Loc, Str>) {
        let old_t = self.t.clone();
        let old_constuctor = self.constructor.clone();

        self.find_in_access_chain_rec(e);

        self.t = old_t;
        self.constructor = old_constuctor;
    }

    fn find_in_access_chain_rec(&mut self, e: &Expression<Loc, Str>) {
        match &e.node {
            ExpressionNode::OpCall(op) => {
                if let OpCall::Unary(UnaryOp::Access(s), rhs) = &op {
                    self.find_in_access_chain_rec(rhs);
                    self.check_add(s);
                    self.apply_variable(s);
                    return;
                }
                panic!("bad access chain")
            }
            ExpressionNode::FunCall { fun, args } => {
                if !args.is_empty() {
                    panic!("bad access chain");
                }

                self.check_add(fun);
                self.apply_variable(fun);
            }
            _ => {
                panic!("bad access chain");
            }
        }
    }
}
