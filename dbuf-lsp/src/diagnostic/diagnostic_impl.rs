use dbuf_core::location::{Location, Offset};
use tower_lsp::lsp_types::Diagnostic;
use tower_lsp::lsp_types::DiagnosticSeverity;

use crate::core::ast_access::File;
use crate::core::ast_access::LocationHelper;

use crate::core::errors::Error;
use dbuf_core::error::Error as CoreError;
use dbuf_core::error::lexing::Error as LexingError;
use dbuf_core::error::parsing::Error as ParsingError;

pub(super) fn provide_diagnostic(f: &File) -> Vec<Diagnostic> {
    let errors = f.get_errors();

    let mut ans = vec![];

    for e in errors {
        let Error::Compiler(e) = e else {
            continue;
        };

        match e {
            CoreError::LexingError(error) => {
                ans.push(convert_lexing_error(error));
            }
            CoreError::ParsingError(error) => ans.push(convert_parsing_error(error)),
            CoreError::ElaboratingError(_) => todo!(),
        }
    }

    ans
}

fn convert_lexing_error(err: &LexingError) -> Diagnostic {
    let range_from = err.data.at;
    let range_len = err.data.current.len();
    let location = Location {
        start: range_from,
        length: Offset {
            lines: 0,
            columns: range_len,
        },
    };

    Diagnostic {
        range: location.to_lsp(),
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: None,
        message: err.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}

fn convert_parsing_error(err: &ParsingError) -> Diagnostic {
    Diagnostic {
        range: err.at.to_lsp(),
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: None,
        message: err.to_string(),
        related_information: None,
        tags: None,
        data: None,
    }
}
