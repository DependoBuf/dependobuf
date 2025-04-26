use std::collections::{BTreeMap, BTreeSet};

use super::operators::OpCall;

/// An elaborated DependoBuf module.
#[derive(Debug)]
pub struct Module<Str> {
    /// List of elaborated types in topologically sorted order.
    pub types: Vec<(Str, Type<Str>)>,
    /// Collection of elaborated constructors for types.
    pub constructors: BTreeMap<Str, Constructor<Str>>,
}

/// Elaborated DependoBuf type.
#[derive(Debug)]
pub struct Type<Str> {
    /// List of elaborated dependencies.
    pub dependencies: Context<Str>,
    /// List of elaborated constructors' names.
    pub constructor_names: ConstructorNames<Str>,
}

/// Constructor names of a type.
#[derive(Debug)]
pub enum ConstructorNames<Str> {
    /// Message has a single constructor.
    OfMessage(Str),
    /// Enum has several constructors.
    OfEnum(BTreeSet<Str>),
}

/// Elaborated DependoBuf constructor.
#[derive(Debug)]
pub struct Constructor<Str> {
    /// List of elaborated implicit arguments' types.
    pub implicits: Context<Str>,
    /// List of elaborated explicit fields' types.
    pub fields: Context<Str>,
    /// Elaborated result type.
    pub result_type: TypeExpression<Str>,
}

/// Context is a list of typed variables.
pub type Context<Str> = Vec<(Str, TypeExpression<Str>)>;

/// Elaborated DependoBuf expression returning value.
#[derive(Clone, Debug)]
pub enum ValueExpression<Str> {
    /// An operator call.
    OpCall {
        op_call: OpCall<Str, Rec<ValueExpression<Str>>>,
        result_type: TypeExpression<Str>,
    },
    /// Call to constructor.
    Constructor {
        name: Str,
        implicits: ValueExprs<Str>,
        arguments: ValueExprs<Str>,
        result_type: TypeExpression<Str>,
    },
    /// Just a variable.
    Variable { name: Str, ty: TypeExpression<Str> },
}

/// Elaborated DependoBuf expression returning type.
#[derive(Clone, Debug)]
pub enum TypeExpression<Str> {
    /// Call to dependent type.
    TypeExpression {
        name: Str,
        dependencies: ValueExprs<Str>,
    },
}

/// Shared list of value subexpressions. There are no type subexpressions for now.
pub type ValueExprs<S> = Rec<[ValueExpression<S>]>;

/// Expression uses Rc for recursion.
/// Consider migrating to Arc when going multicore.
pub type Rec<T> = std::rc::Rc<T>;
