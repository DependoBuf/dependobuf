//! Module contains `ElaboratingStage` struct - error data for elaborating stage.

use thiserror::Error;

use super::ErrorStage;

#[derive(Debug)]
enum Void {}

/// TODO: implement
#[derive(Debug, Error)]
#[error("Elaborating stage")]
pub struct ElaboratingStage {
    unconstructable: Void,
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
