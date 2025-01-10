use crate::expression::{Context, Expression};
use std::collections::BTreeMap;

/// An elaborated DependoBuf module defines a list of named types
/// in their topologically sorted order.
pub type ElaboratedModule<Name> = Vec<(Name, ElaboratedType<Name>)>;

/// Elaborated DependoBuf type with a list of constructors.
#[derive(Debug)]
pub struct ElaboratedType<Name> {
    /// List of elaborated dependencies.
    pub dependencies: Context<Name>,
    /// List of elaborated constructors.
    pub constructors: BTreeMap<Name, ElaboratedConstructor<Name>>,
}

/// Elaborated DependoBuf constructor.
#[derive(Debug)]
pub struct ElaboratedConstructor<Name> {
    /// List of elaborated implicit arguments' types.
    pub implicits: Context<Name>,
    /// List of elaborated explicit fields' types.
    pub fields: Context<Name>,
    /// List of corresponding elaborated dependencies of a result type.
    pub result_dependencies: Vec<Expression<Name>>,
}
