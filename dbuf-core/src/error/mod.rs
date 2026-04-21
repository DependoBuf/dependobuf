//! Module contains `Error` struct that contains
//! every possible error can appear during compilation.

pub mod elaborating;
pub mod lexing;
pub mod parsing;

use std::{fmt::Display, ops::Deref};

use thiserror::Error;

use elaborating::ElaboratingStage;
use lexing::LexingStage;
use parsing::ParsingStage;

use crate::location::{Location, Offset};

/// Stages of project pipeline.
pub trait ErrorStage: Display {
    /// location of error
    fn location(&self) -> Location<Offset>;
}

/// Error struct for every project error.
///
/// Param:
/// * `Stage` is the last compilation stage completed.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub struct Error<Stage> {
    pub stage: Stage,
}

pub type LexingError = Error<LexingStage>;
pub type ParsingError = Error<ParsingStage>;
pub type ElaboratingError = Error<ElaboratingStage>;

/// Error enum, that contains all possible errors.
#[derive(Debug, Error)]
pub enum GeneralError {
    #[error("Lexing error {}", .0)]
    Lexing(#[from] LexingError),
    #[error("Parsing error {}", .0)]
    Parsing(#[from] ParsingError),
    #[error("Elaborating error {}", .0)]
    Elaborating(#[from] ElaboratingError),
}

impl<Stage: ErrorStage> Display for Error<Stage> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.stage.fmt(f)
    }
}

impl From<LexingStage> for Error<LexingStage> {
    fn from(value: LexingStage) -> Self {
        Error { stage: value }
    }
}

impl From<ParsingStage> for Error<ParsingStage> {
    fn from(value: ParsingStage) -> Self {
        Error { stage: value }
    }
}

impl From<ElaboratingStage> for Error<ElaboratingStage> {
    fn from(value: ElaboratingStage) -> Self {
        Error { stage: value }
    }
}

impl<T: ErrorStage> Deref for Error<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.stage
    }
}
