use std::collections::HashSet;
use std::hash::Hash;
use crate::ast::elaborated::{
    Constructor, Context, Module, Rec, TypeExpression, ValueExpression, Type,
    ValueExprs,
};

pub trait Rename: Sized {
    fn with_suffix(&self, suffix: &str) -> Self;
}

impl Rename for String {
    fn with_suffix(&self, suffix: &str) -> Self {
        format!("{self}{suffix}")
    }
}

impl Rename for crate::arena::InternedString {
    fn with_suffix(&self, suffix: &str) -> Self {
        Self::from(format!("{}{suffix}", self.as_ref()))
    }
}

pub fn map_module<A, B, F>(module: Module<A>, f: &F) -> Module<B>
where
    F: Fn(A) -> B,
    A: Clone,
    B: Clone + Ord,
{
    Module {
        constructors: module
            .constructors
            .into_iter()
            .map(|(name, ctor)| (f(name.clone()), map_constructor(ctor, f)))
            .collect(),
        types: module
            .types
            .into_iter()
            .map(|(name, ty)| (f(name.clone()), map_type(ty, f)))
            .collect(),
    }
}

fn map_type<A, B, F>(ty: Type<A>, f: &F) -> Type<B>
where
    F: Fn(A) -> B,
    A: Clone,
    B: Clone + Ord,
{
    Type {
        dependencies: map_context(ty.dependencies, f),
        constructor_names: match ty.constructor_names {
            crate::ast::elaborated::ConstructorNames::OfMessage(name) => {
                crate::ast::elaborated::ConstructorNames::OfMessage(f(name))
            }
            crate::ast::elaborated::ConstructorNames::OfEnum(names) => {
                crate::ast::elaborated::ConstructorNames::OfEnum(names.into_iter().map(f).collect())
            }
        },
    }
}

fn map_constructor<A, B, F>(ctor: Constructor<A>, f: &F) -> Constructor<B>
where
    F: Fn(A) -> B,
    A: Clone,
{
    Constructor {
        implicits: map_context(ctor.implicits, f),
        fields: map_context(ctor.fields, f),
        result_type: map_type_expression(ctor.result_type, f),
    }
}

fn map_context<A, B, F>(ctx: Context<A>, f: &F) -> Context<B>
where
    F: Fn(A) -> B,
    A: Clone,
{
    ctx.into_iter()
        .map(|(name, ty)| (f(name), map_type_expression(ty, f)))
        .collect()
}

fn map_type_expression<A, B, F>(ty: TypeExpression<A>, f: &F) -> TypeExpression<B>
where
    F: Fn(A) -> B,
    A: Clone,
{
    match ty {
        TypeExpression::TypeExpression { name, dependencies } => TypeExpression::TypeExpression {
            name: f(name),
            dependencies: map_value_exprs(dependencies, f),
        },
    }
}

fn map_value_exprs<A, B, F>(exprs: ValueExprs<A>, f: &F) -> ValueExprs<B>
where
    F: Fn(A) -> B,
    A: Clone,
{
    Rec::from(
        exprs
            .iter()
            .cloned()
            .map(|e| map_value_expression(e, f))
            .collect::<Vec<_>>(),
    )
}

fn map_value_expression<A, B, F>(expr: ValueExpression<A>, f: &F) -> ValueExpression<B>
where
    F: Fn(A) -> B,
    A: Clone,
{
    match expr {
        ValueExpression::Variable { name, ty } => ValueExpression::Variable {
            name: f(name),
            ty: map_type_expression(ty, f),
        },
        ValueExpression::Constructor { name, implicits, arguments, result_type } => {
            ValueExpression::Constructor {
                name: f(name),
                implicits: map_value_exprs(implicits, f),
                arguments: map_value_exprs(arguments, f),
                result_type: map_type_expression(result_type, f),
            }
        }
        ValueExpression::OpCall { op_call, result_type } => ValueExpression::OpCall {
            op_call: map_op_call(op_call, f),
            result_type: map_type_expression(result_type, f),
        },
    }
}

fn map_op_call<A, B, F>(
    op_call: crate::ast::operators::OpCall<A, Rec<ValueExpression<A>>>,
    f: &F,
) -> crate::ast::operators::OpCall<B, Rec<ValueExpression<B>>>
where
    F: Fn(A) -> B,
    A: Clone,
{
    use crate::ast::operators::OpCall;
    match op_call {
        OpCall::Literal(lit) => OpCall::Literal(lit),
        OpCall::Unary(op, expr) => {
            let op = match op {
                crate::ast::operators::UnaryOp::Access(name) => {
                    crate::ast::operators::UnaryOp::Access(f(name))
                }
                crate::ast::operators::UnaryOp::Minus => crate::ast::operators::UnaryOp::Minus,
                crate::ast::operators::UnaryOp::Bang => crate::ast::operators::UnaryOp::Bang,
            };
            OpCall::Unary(op, Rec::new(map_value_expression((*expr).clone(), f)))
        }
        OpCall::Binary(op, lhs, rhs) => OpCall::Binary(
            op,
            Rec::new(map_value_expression((*lhs).clone(), f)),
            Rec::new(map_value_expression((*rhs).clone(), f)),
        ),
    }
}

pub fn add_suffix_context<Str>(context: Context<Str>, suffix: &str) -> Context<Str>
where
    Str: Clone + Eq + Hash + Rename,
{
    let implicit_names: HashSet<Str> = context.iter().map(|(name, _)| name.clone()).collect();

    let f = |name: Str| {
        if implicit_names.contains(&name) {
            name.with_suffix(suffix)
        } else {
            name
        }
    };

    map_context(context, &f)
}
