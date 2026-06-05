pub mod from_elaborated;
mod node_id;

pub use dbuf_core::arena::InternedString;
pub use dbuf_core::ast::{
    elaborated,
    operators::{BinaryOp, Literal},
};
pub use node_id::NodeId;

use node_id::Node;

use std::rc::{Rc, Weak};

pub type Str = InternedString;

pub struct Module {
    pub types: Vec<Rc<Type>>,
    pub _builtins: Vec<Rc<Type>>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum TypeKind {
    Message,
    Enum,
}

#[derive(Clone)]
pub struct Type {
    pub name: Str,
    pub dependencies: Vec<Rc<Symbol>>,
    pub constructors: Vec<Rc<Constructor>>,
    pub kind: TypeKind,
    pub is_builtin: bool,
}

#[derive(Clone)]
pub struct Constructor {
    pub name: Str,
    pub implicits: Vec<Rc<Symbol>>,
    pub fields: Vec<Rc<Symbol>>,
    pub result_type: TypeExpression,
}

#[derive(Clone)]
pub enum ValueExpression {
    OpCall(OpCall),
    Constructor {
        call: Weak<Constructor>,
        implicits: Vec<ValueExpression>,
        arguments: Vec<ValueExpression>,
    },
    Variable(Weak<Symbol>),
}

#[derive(Clone)]
pub enum TypeExpression {
    Type {
        call: Weak<Type>,
        dependencies: Vec<ValueExpression>,
    },
}

#[derive(Clone)]
pub struct Symbol {
    pub name: Str,
    pub ty: TypeExpression,
}

#[derive(Clone)]
pub enum OpCall {
    Literal(Literal),
    Unary(UnaryOp, Box<ValueExpression>),
    Binary(BinaryOp, Box<ValueExpression>, Box<ValueExpression>),
}

#[derive(Clone)]
pub enum UnaryOp {
    Access { to: Weak<Type>, field: Weak<Symbol> },
    Minus,
    Bang,
}

impl Node for Module {}
impl Node for Type {}
impl Node for Constructor {}
impl Node for ValueExpression {}
impl Node for TypeExpression {}
impl Node for Symbol {}

impl TypeExpression {
    pub fn get_type(&self) -> Rc<Type> {
        match self {
            TypeExpression::Type {
                call,
                dependencies: _,
            } => call.upgrade().expect("call to unknown type"),
        }
    }

    pub fn get_dependencies(&self) -> &Vec<ValueExpression> {
        match self {
            TypeExpression::Type {
                call: _,
                dependencies,
            } => dependencies,
        }
    }
}
