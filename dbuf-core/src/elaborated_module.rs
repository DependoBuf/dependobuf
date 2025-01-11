use crate::expression::{Context, TypeExpression};
use std::collections::{BTreeMap, BTreeSet};

/// An elaborated DependoBuf module.
#[derive(Debug)]
pub struct ElaboratedModule<Name> {
    /// List of elaborated types in topologically sorted order.
    pub types: Vec<(Name, ElaboratedType<Name>)>,
    /// Collection of elaborated constructors for types.
    pub constructors: BTreeMap<Name, ElaboratedConstructor<Name>>,
}

/// Elaborated DependoBuf type.
#[derive(Debug)]
pub struct ElaboratedType<Name> {
    /// List of elaborated dependencies.
    pub dependencies: Context<Name>,
    /// List of elaborated constructors' names.
    pub constructor_names: BTreeSet<Name>,
}

/// Elaborated DependoBuf constructor.
#[derive(Debug)]
pub struct ElaboratedConstructor<Name> {
    /// List of elaborated implicit arguments' types.
    pub implicits: Context<Name>,
    /// List of elaborated explicit fields' types.
    pub fields: Context<Name>,
    /// Elaborated result type.
    pub result_type: TypeExpression<Name>,
}
