pub mod definition;

use super::operators::{Literal, OpCall};
use definition::Definitions;

/// A single DependoBuf module is a list of type definitions.
pub type Module<Loc, Str> = Definitions<Loc, Str, TypeDeclaration<Loc, Str>>;

/// Declaration of a DependoBuf type.
#[derive(Debug)]
pub struct TypeDeclaration<Loc, Str> {
    /// List of dependencies & their types.
    pub dependencies: Definitions<Loc, Str, TypeExpression<Loc, Str>>,
    /// Definition.
    pub body: TypeDefinition<Loc, Str>,
}

/// Definition of a DependoBuf type.
#[derive(Debug)]
pub enum TypeDefinition<Loc, Str> {
    /// Message has a single constructor.
    Message(ConstructorBody<Loc, Str>),
    /// Enum can have several branches.
    Enum(Vec<EnumBranch<Loc, Str>>),
}

/// Single branch of a DependoBuf enum type.
#[derive(Debug)]
pub struct EnumBranch<Loc, Str> {
    /// List of patterns to match dependencies against.
    pub patterns: Vec<Pattern<Loc, Str>>,
    /// List of available constructors on successful pattern match.
    pub constructors: Definitions<Loc, Str, ConstructorBody<Loc, Str>>,
}

/// Constructor body is a list of typed variables.
pub type ConstructorBody<Loc, Str> = Definitions<Loc, Str, TypeExpression<Loc, Str>>;

/// Type expression is just an expression returning a type.
pub type TypeExpression<Loc, Str> = Expression<Loc, Str>;

/// Each parsed subexpression has a location.
#[derive(Clone, Debug)]
pub struct Expression<Loc, Str> {
    /// Location of a subexpression.
    pub loc: Loc,
    /// Subexpression itself.
    pub node: ExpressionNode<Loc, Str, Expression<Loc, Str>>,
}

/// Possible expression node types.
#[derive(Clone, Debug)]
pub enum ExpressionNode<Loc, Str, Expr> {
    /// Operator call.
    OpCall(OpCall<Str, Rec<Expr>>),
    /// Call to dependent type.
    FunCall { fun: Str, args: Rec<[Expr]> },
    /// Constructor call.
    ConstructorCall {
        name: Str,
        fields: Definitions<Loc, Str, Expr>,
    },
    /// Variable.
    Variable { name: Str },
    /// Typed hole which should report expected type of a missing expression.
    TypedHole,
}

/// Each parsed subpattern has a location.
#[derive(Clone, Debug)]
pub struct Pattern<Loc, Str> {
    /// Location of a subpattern.
    pub loc: Loc,
    /// Subpattern itself.
    pub node: PatternNode<Loc, Str, Pattern<Loc, Str>>,
}

/// Possible pattern node types.
#[derive(Clone, Debug)]
pub enum PatternNode<Loc, Str, Pattern> {
    /// Constructor call.
    ConstructorCall {
        name: Str,
        fields: Definitions<Loc, Str, Pattern>,
    },
    /// Variable.
    Variable { name: Str },
    /// Literal.
    Literal(Literal),
    /// A catch-all pattern.
    Underscore,
}

/// Parsed expressions and patterns use Arc for recursion.
pub type Rec<T> = std::sync::Arc<T>;
