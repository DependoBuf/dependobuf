use tower_lsp::lsp_types::Diagnostic;
use tower_lsp::lsp_types::DiagnosticRelatedInformation;
use tower_lsp::lsp_types::DiagnosticSeverity;
use tower_lsp::lsp_types::Location;
use tower_lsp::lsp_types::Url;

use crate::core::workspace::File;
use crate::core::workspace::Loc;
use crate::core::workspace::LocationHelper;

use crate::core::errors::Error;
use dbuf_core::error::ElaboratingError;
use dbuf_core::error::Error as CoreError;
use dbuf_core::error::ErrorStage;
use dbuf_core::error::ParsingError;
use dbuf_core::error::elaborating as e;
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
            Error::Parsing(error) => ans.push(error.convert(uri)),
            Error::ElaboratingError(error) => ans.push(error.convert(uri)),
        }
    }
    ans
}

trait Convertable {
    fn convert(&self, uri: &Url) -> Diagnostic;
}

impl<S: ErrorStage> Convertable for CoreError<S>
where
    CoreError<S>: Buildable<()>,
{
    fn convert(&self, uri: &Url) -> Diagnostic {
        let at = self.location();
        self.build(&()).finish(at, uri)
    }
}

struct ErrorBuilder {
    severity: DiagnosticSeverity,
    message: String,
    related_info: Vec<(String, Loc)>,
}

trait Buildable<Parent> {
    fn build(&self, parent: &Parent) -> ErrorBuilder;
}

impl Buildable<ParsingError> for BadCallChain {
    fn build(&self, parent: &ParsingError) -> ErrorBuilder {
        assert!(matches!(parent.extra, Some(ErrorExtra::BadCallChain(_))));

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

impl Buildable<ParsingError> for MissingComma {
    fn build(&self, parent: &ParsingError) -> ErrorBuilder {
        assert!(matches!(parent.extra, Some(ErrorExtra::MissingComma(_))));

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

impl Buildable<ParsingError> for TypedHole {
    fn build(&self, parent: &ParsingError) -> ErrorBuilder {
        assert!(matches!(parent.extra, Some(ErrorExtra::TypedHole(_))));

        let message = "Unresolved type hole found".to_owned();
        let related_info = vec![];

        ErrorBuilder {
            severity: DiagnosticSeverity::WARNING,
            message,
            related_info,
        }
    }
}

impl Buildable<ParsingError> for ParserLexingError {
    fn build(&self, parent: &ParsingError) -> ErrorBuilder {
        assert!(matches!(parent.extra, Some(ErrorExtra::LexingError(_))));

        let message = format!("Lexing error: {}", self.0);
        let related_info = vec![];

        ErrorBuilder {
            severity: DiagnosticSeverity::WARNING,
            message,
            related_info,
        }
    }
}

impl Buildable<ParsingError> for () {
    fn build(&self, parent: &ParsingError) -> ErrorBuilder {
        assert!(parent.extra.is_none());

        let found_str = if let Some(t) = &parent.found {
            format!("Unexpected token {t}.")
        } else {
            "Unexpected token.".to_owned()
        };

        let mut expected_str = String::new();
        if !&parent.expected.is_empty() {
            expected_str += " Expected:";
            for expect in &parent.expected {
                if expect.is_internal() {
                    continue;
                }
                let _ = write!(expected_str, " '{expect}'");
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

impl Buildable<()> for ParsingError {
    fn build(&self, &(): &()) -> ErrorBuilder {
        match &self.extra {
            Some(ErrorExtra::BadCallChain(e)) => e.build(self),
            Some(ErrorExtra::MissingComma(e)) => e.build(self),
            Some(ErrorExtra::TypedHole(e)) => e.build(self),
            Some(ErrorExtra::LexingError(e)) => e.build(self),
            None => ().build(self),
        }
    }
}

impl Buildable<()> for ElaboratingError {
    fn build(&self, &(): &()) -> ErrorBuilder {
        let message = self.stage.error.to_string();

        let mut related_info = vec![];

        match &self.stage.error {
            e::Error::Cycle(entries) => {
                related_info = entries
                    .iter()
                    .map(|(name, loc)| (format!("{name} is part of the cycle"), *loc))
                    .collect();
            }
            e::Error::NoInitialConstructor(entries) => {
                related_info = entries
                    .iter()
                    .map(|(name, loc)| (format!("{name} has no initial constructor"), *loc))
                    .collect();
            }
            _ => {}
        }

        ErrorBuilder {
            severity: DiagnosticSeverity::ERROR,
            message,
            related_info,
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
