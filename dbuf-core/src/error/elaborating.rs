//! Module contains `elaborating::Error` - errors that appear
//! during elaborating phase

use super::ErrorStage;
use crate::arena::InternedString;
use crate::ast::elaborated;
use crate::ast::operators::Literal;
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
    #[error("cycle in type dependencies")]
    Cycle(Vec<String>),
    #[error("no initial constructor for: {}", .0.join(", "))]
    NoInitialConstructor(Vec<String>),
    #[error("type hole should have type {0:?}")]
    TypeHole(elaborated::TypeExpression<InternedString>),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("elaborating error: {error}")]
pub struct ElaboratingStage {
    pub error: Error,
}

impl ErrorStage for ElaboratingStage {
    fn location(&self) -> crate::location::Location<crate::location::Offset> {
        unimplemented!("Elaborating stage location stores no location")
    }
}

impl From<super::Error<ElaboratingStage>> for ElaboratingStage {
    fn from(value: super::Error<ElaboratingStage>) -> Self {
        value.stage
    }
}

impl From<Error> for super::ElaboratingError {
    fn from(value: Error) -> Self {
        Self {
            stage: ElaboratingStage { error: value },
        }
    }
}
