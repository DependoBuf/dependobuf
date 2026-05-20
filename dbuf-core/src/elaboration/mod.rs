use crate::arena::InternedString;
use crate::ast::{elaborated as e, operators as o, parsed as p};
use crate::error::elaborating::Error;
use crate::location::{LocatedName, Location, Offset};

pub mod apply;
pub mod builtins;
pub mod context;
pub mod graph;
pub mod normalize;
pub mod operators;
pub mod rename;
pub mod subst;
pub mod typecheck;
pub mod unify;

type Str = InternedString;
type Loc = Location<Offset>;
type Name = LocatedName<Str, Offset>;
type TypeExpr = e::TypeExpression<Str>;
type Value = e::ValueExpression<Str>;
type ElaboratedCtx = e::Context<Str>;
type Mod = e::Module<Str>;
type Ctx<'a> = context::Context<'a, Str, TypeExpr, Value>;
type ElaboratedDeclaration = (Str, e::Type<Str>, Vec<(Str, e::Constructor<Str>)>);
type Binds<Str> = Vec<(Str, e::ValueExpression<Str>)>;
type Bindings = Binds<Str>;

impl From<&o::UnaryOp<Name>> for o::UnaryOp<Str> {
    fn from(value: &o::UnaryOp<Name>) -> Self {
        match value {
            o::UnaryOp::Access(s) => o::UnaryOp::Access(s.content.clone()),
            o::UnaryOp::Minus => o::UnaryOp::Minus,
            o::UnaryOp::Bang => o::UnaryOp::Bang,
        }
    }
}

/// # Errors
pub fn elaborate(module: &p::Module<Loc, Name>) -> Result<Mod, Error> {
    let sorted = graph::topological_sort(module)?;

    let missing = graph::check_initial_constructors(module);
    if !missing.is_empty() {
        return Err(Error::NoInitialConstructor(missing));
    }

    typecheck::elaborate_sorted(&sorted)
}

fn type_of<Str: Clone>(expr: &e::ValueExpression<Str>) -> e::TypeExpression<Str> {
    match expr {
        e::ValueExpression::Variable { ty, .. } => ty.clone(),
        e::ValueExpression::Constructor { result_type, .. } => result_type.clone(),
        e::ValueExpression::OpCall { result_type, .. } => result_type.clone(),
    }
}
