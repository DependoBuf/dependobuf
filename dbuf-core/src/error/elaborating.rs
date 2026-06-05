//! Module contains `elaborating::Error` - errors that appear
//! during elaborating phase

use super::ErrorStage;
use crate::arena::InternedString;
use crate::ast::elaborated;
use crate::ast::operators::Literal;
use crate::location::{Location, Offset};
use thiserror::Error;

/// Errors that can occur during type elaboration.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("unknown type {0}")]
    UnknownType(String),
    #[error("unknown variable {0}")]
    UnknownVariable(String),
    #[error("unknown constructor {0}")]
    UnknownConstructor(String),
    #[error("unknown field {0}")]
    UnknownField(String),
    #[error("arity mismatch: expected {expected}, found {found}")]
    ArityMismatch { expected: usize, found: usize },
    #[error("type mismatch")]
    TypeMismatch,
    #[error("operator type mismatch")]
    OperatorTypeMismatch,
    #[error("unsupported syntax")]
    UnsupportedSyntax,
    #[error("constructor mismatch: {0} vs {1}")]
    ConstructorMismatch(String, String),
    #[error("literal mismatch: {0:?} vs {1:?}")]
    LiteralMismatch(Literal, Literal),
    #[error("conflicting binding for {0}")]
    ConflictingBinding(String),
    #[error("cycle in type dependencies: {}", .0.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(" -> "))]
    Cycle(Vec<(String, Location<Offset>)>),
    #[error("no initial constructor for: {}", .0.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", "))]
    NoInitialConstructor(Vec<(String, Location<Offset>)>),
    #[error("type hole should have type {0:?}")]
    TypeHole(elaborated::TypeExpression<InternedString>),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("elaborating error: {error}")]
pub struct ElaboratingStage {
    pub error: Error,
    pub loc: Option<Location<Offset>>,
}

impl ErrorStage for ElaboratingStage {
    fn location(&self) -> Location<Offset> {
        self.loc.unwrap_or_default()
    }
}

impl From<super::Error<ElaboratingStage>> for ElaboratingStage {
    fn from(value: super::Error<ElaboratingStage>) -> Self {
        value.stage
    }
}
