//! Module contains labels for parser
//!

use std::fmt::Display;

use strum::EnumMessage;
use strum_macros::EnumMessage;

/// Label for parser expections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumMessage)]
pub enum Label {
    /// Body
    Body,
    /// Field
    Field,
    /// Definition
    Definition,
    /// Expression
    Expression,
    /// Parened Expression
    ParenedExpression,
    /// Type Indentifier
    TypeIndentifier,
    /// Variable Identifier
    VariableIdentifier,
    /// Literal
    Literal,
    /// Typed Hole
    TypedHole,
    /// Space
    Space,
    /// New Line
    NewLine,
    /// Error
    Error,
    /// Comment
    Comment,
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.get_documentation()
                .expect("every variant has documentation")
        )
    }
}
