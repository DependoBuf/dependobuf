//! Module contains `elaborating::Error` - errors that appear
//! during elaborating phase

use super::ErrorStage;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// TODO: implement
#[derive(Debug, Error)]
pub enum Error {
    ElaboratingError,
}

impl Display for Error {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!("error display not implemented")
    }
}

#[derive(Debug, Error)]
#[error("Elaborating stage")]
pub struct ElaboratingStage {
    error: Error,
}

impl ErrorStage for ElaboratingStage {
    fn location(&self) -> crate::location::Location<crate::location::Offset> {
        unreachable!("since ElaboratingStage is unconstructable")
    }
}

impl From<super::Error<ElaboratingStage>> for ElaboratingStage {
    fn from(value: super::Error<ElaboratingStage>) -> Self {
        value.stage
    }
}
