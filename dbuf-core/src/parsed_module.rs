use crate::expression::{Context, Pattern};

/// A single DependoBuf module is a list of type declarations.
pub type ParsedModule<Name> = Vec<(Name, TypeDeclaration<Name>)>;

/// Declaration of a DependoBuf type.
#[derive(Debug)]
pub struct TypeDeclaration<Name> {
    /// List of dependencies.
    pub dependencies: Context<Name>,
    /// Definition.
    pub body: TypeDefinition<Name>,
}

/// Definition of a DependoBuf type.
#[derive(Debug)]
pub enum TypeDefinition<Name> {
    /// Message has a single constructor.
    Message(ConstructorBody<Name>),
    /// Enum can have several branches.
    Enum(Vec<EnumBranch<Name>>),
}

/// Constructor body is a list of typed variables.
pub type ConstructorBody<Name> = Context<Name>;

/// Single branch of a DependoBuf enum type.
#[derive(Debug)]
pub struct EnumBranch<Name> {
    /// List of pattern to match dependencies against.
    pub patterns: Vec<Pattern<Name>>,
    /// List of available constructors on successful pattern match.
    pub constructors: Vec<(Name, ConstructorBody<Name>)>,
}
