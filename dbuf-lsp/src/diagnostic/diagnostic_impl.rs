use tower_lsp::lsp_types::Diagnostic;
use tower_lsp::lsp_types::DiagnosticRelatedInformation;
use tower_lsp::lsp_types::DiagnosticSeverity;
use tower_lsp::lsp_types::Location;
use tower_lsp::lsp_types::Url;

use crate::core::ast_access::File;
use crate::core::ast_access::Loc;
use crate::core::ast_access::LocationHelper;

use crate::core::errors::Error;
use dbuf_core::error::ErrorStage;
use dbuf_core::error::ParsingError;
use dbuf_core::error::parsing::*;

use std::fmt::Write as _;

pub(super) fn provide_diagnostic(f: &File) -> Vec<Diagnostic> {
    let errors = f.get_errors();

    let mut ans = vec![];
    let uri = f.get_uri();
    for e in errors {
        match e {
            Error::Format(_format_error) => (),
            Error::Rename(_rename_error) => (),
            Error::Parsing(error) => ans.push(convert_parsing_error(error, uri)),
            Error::ElaboratingError(_error) => todo!(),
        }
    }
    ans
}

fn convert_parsing_error(err: &ParsingError, uri: &Url) -> Diagnostic {
    let at = err.location();
    err.build(err).finish(at, uri)
}

struct ErrorBuilder {
    severity: DiagnosticSeverity,
    message: String,
    related_info: Vec<(String, Loc)>,
}

trait Buildable {
    fn build(&self, err: &ParsingError) -> ErrorBuilder;
}

impl Buildable for BadCallChain {
    fn build(&self, err: &ParsingError) -> ErrorBuilder {
        assert!(matches!(err.extra, Some(ErrorExtra::BadCallChain(_))));

        let message = "Call chain ends with dot".to_owned();

        let extra_message = "Call chain".to_owned();
        let extra_loc = self.0;
        let related_info = vec![(extra_message, extra_loc)];

        ErrorBuilder {
            severity: DiagnosticSeverity::WARNING,
            message,
            related_info,
        }
    }
}

impl Buildable for MissingComma {
    fn build(&self, err: &ParsingError) -> ErrorBuilder {
        assert!(matches!(err.extra, Some(ErrorExtra::MissingComma(_))));

        let message = "Line missing ending semicolon".to_owned();

        let extra_message = "Line".to_owned();
        let extra_loc = self.0;
        let related_info = vec![(extra_message, extra_loc)];

        ErrorBuilder {
            severity: DiagnosticSeverity::WARNING,
            message,
            related_info,
        }
    }
}

impl Buildable for TypedHole {
    fn build(&self, err: &ParsingError) -> ErrorBuilder {
        assert!(matches!(err.extra, Some(ErrorExtra::TypedHole(_))));

        let message = "Unresolved type hole found".to_owned();
        let related_info = vec![];

        ErrorBuilder {
            severity: DiagnosticSeverity::WARNING,
            message,
            related_info,
        }
    }
}

impl Buildable for ParserLexingError {
    fn build(&self, err: &ParsingError) -> ErrorBuilder {
        assert!(matches!(err.extra, Some(ErrorExtra::LexingError(_))));

        let message = format!("Lexing error: {}", self.0);
        let related_info = vec![];

        ErrorBuilder {
            severity: DiagnosticSeverity::WARNING,
            message,
            related_info,
        }
    }
}

impl Buildable for () {
    fn build(&self, err: &ParsingError) -> ErrorBuilder {
        assert!(err.extra.is_none());

        let found_str = if let Some(t) = &err.found {
            format!("Unexpected token {t}.")
        } else {
            "Unexpected token.".to_owned()
        };

        let mut expected_str = String::new();
        if !&err.expected.is_empty() {
            expected_str += " Expected:";
            for expect in &err.expected {
                let _ = write!(expected_str, " '{expect}");
            }
            expected_str += ".";
        }

        let message = found_str + &expected_str;
        let related_info = vec![];

        ErrorBuilder {
            severity: DiagnosticSeverity::ERROR,
            message,
            related_info,
        }
    }
}

impl Buildable for ParsingError {
    fn build(&self, err: &ParsingError) -> ErrorBuilder {
        match &err.extra {
            Some(ErrorExtra::BadCallChain(e)) => e.build(err),
            Some(ErrorExtra::MissingComma(e)) => e.build(err),
            Some(ErrorExtra::TypedHole(e)) => e.build(err),
            Some(ErrorExtra::LexingError(e)) => e.build(err),
            None => ().build(err),
        }
    }
}

impl ErrorBuilder {
    fn finish(self, at: Loc, uri: &Url) -> Diagnostic {
        let related_information = self
            .related_info
            .into_iter()
            .map(|(msg, loc)| DiagnosticRelatedInformation {
                location: Location {
                    uri: uri.clone(),
                    range: loc.to_lsp(),
                },
                message: msg,
            })
            .collect();

        Diagnostic {
            range: at.to_lsp(),
            severity: Some(self.severity),
            code: None,
            code_description: None,
            source: None,
            message: self.message,
            related_information: Some(related_information),
            tags: None,
            data: None,
        }
    }
}
