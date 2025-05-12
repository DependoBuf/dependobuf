//! `textDocument/format` helpers.
//!

use tower_lsp::lsp_types::FormattingOptions;

use crate::core::errors::FormatError;

trait BoolCheck {
    fn check(self, err: FormatError) -> Result<(), FormatError>;
}

impl BoolCheck for bool {
    fn check(self, err: FormatError) -> Result<(), FormatError> {
        self.then_some(()).ok_or(err)
    }
}

pub fn valid_options(options: &FormattingOptions) -> Result<(), FormatError> {
    options.insert_spaces.check(FormatError::InsertSpaces)?;
    options
        .properties
        .is_empty()
        .check(FormatError::Properties)?;
    options
        .trim_trailing_whitespace
        .is_none()
        .check(FormatError::TrimTrailingWhitespace)?;
    options
        .insert_final_newline
        .is_none()
        .check(FormatError::InsertFinalNewLine)?;
    options
        .trim_final_newlines
        .is_none()
        .check(FormatError::TrimFinalNewLines)?;
    Ok(())
}
