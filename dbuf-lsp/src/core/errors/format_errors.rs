use strum::IntoDiscriminant;
use strum_macros::{EnumDiscriminants, EnumIter};

use thiserror::Error;

#[derive(EnumIter, EnumDiscriminants, Error, Debug)]
pub enum FormatError {
    #[error("property 'insert_spaces' not true")]
    InsertSpaces,
    #[error("property 'properties' not empty")]
    Properties,
    #[error("property 'trim_trailing_whitespace' not none")]
    TrimTrailingWhitespace,
    #[error("property 'insert_final_newline' not none")]
    InsertFinalNewLine,
    #[error("property 'trim_final_newlines' not none")]
    TrimFinalNewLines,
}

impl FormatError {
    pub fn get_code(&self) -> i64 {
        self.discriminant() as i64
    }
}
