use std::ops::Range;

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

use dbuf_core::cst::Token;
use dbuf_core::location::Location;
use dbuf_core::location::Offset;

use dbuf_core::error::parsing::*;
use dbuf_core::error::*;

use super::file::File;

pub struct Reporter<'a> {
    meta: Metadata<'a>,
    reports: Vec<Report<'a, (&'a str, Range<usize>)>>,
}

struct Metadata<'a> {
    file: &'a File,
    newlines: Vec<usize>,
}

trait Reportable<Extra> {
    fn report<'a>(&self, meta: &Metadata<'a>, extra: &Extra)
    -> Report<'a, (&'a str, Range<usize>)>;
}

impl<'a> Reporter<'a> {
    /// Create new reporter for file.
    pub fn new(file: &'a File) -> Self {
        Self {
            meta: Metadata::new(file),
            reports: vec![],
        }
    }

    /// Print all reported errors to stderr.
    pub fn print(self) {
        for report in self.reports {
            report
                .eprint((
                    self.meta.file.name.as_ref(),
                    Source::from(&self.meta.file.content),
                ))
                .unwrap();
        }
    }

    /// Report an error.
    pub fn report(&mut self, err: &GeneralError) {
        let r = match err {
            GeneralError::Lexing(error) => error.report(&self.meta, &()),
            GeneralError::Parsing(error) => error.report(&self.meta, &()),
            GeneralError::Elaborating(error) => error.report(&self.meta, &()),
        };
        self.reports.push(r);
    }
}

impl<'a> Metadata<'a> {
    pub fn new(file: &'a File) -> Self {
        let mut newlines = vec![0];
        for (i, ch) in file.content.chars().enumerate() {
            if ch == '\n' {
                newlines.push(i + 1);
            }
        }

        Self { file, newlines }
    }

    fn convert_offset(&self, off: Offset) -> usize {
        if off.lines >= self.newlines.len() {
            self.file.content.len()
        } else {
            self.newlines[off.lines] + off.columns
        }
    }

    fn convert_location(&self, loc: &Location<Offset>) -> Range<usize> {
        let location_start = self.convert_offset(loc.start);
        let location_end = self.convert_offset(loc.end());
        location_start..location_end
    }
}

impl Reportable<()> for LexingError {
    fn report<'a>(&self, meta: &Metadata<'a>, _extra: &()) -> Report<'a, (&'a str, Range<usize>)> {
        let (kind, c) = if self.kind == lexing::ErrorKind::UnknownToken {
            (ReportKind::Warning, Color::Yellow)
        } else {
            (ReportKind::Error, Color::Red)
        };

        let at = self.location();
        let span = meta.convert_location(&at);
        let loc = (meta.file.name.as_ref(), span);

        Report::build(kind, loc.clone())
            .with_message(self.kind.to_string())
            .with_label(Label::new(loc).with_color(c))
            .finish()
    }
}

impl Reportable<ParsingError> for BadCallChain {
    fn report<'a>(
        &self,
        meta: &Metadata<'a>,
        extra: &ParsingError,
    ) -> Report<'a, (&'a str, Range<usize>)> {
        assert!(matches!(extra.extra, Some(ErrorExtra::BadCallChain(_))));
        assert!(extra.found == Some(Token::Dot));

        let kind = ReportKind::Warning;

        let span = meta.convert_location(&extra.at);
        let loc1 = (meta.file.name.as_ref(), span);
        let label1 = Label::new(loc1.clone())
            .with_color(Color::Yellow)
            .with_message(format!("Found {}", (Token::Dot).fg(Color::Yellow)));

        let span = meta.convert_location(&self.0);
        let loc2 = (meta.file.name.as_ref(), span);
        let label2 = Label::new(loc2)
            .with_color(Color::Cyan)
            .with_message("Unfinished call chain");

        Report::build(kind, loc1)
            .with_label(label1)
            .with_label(label2)
            .with_message("Call chain not finished".to_string())
            .finish()
    }
}

impl Reportable<ParsingError> for MissingComma {
    fn report<'a>(
        &self,
        meta: &Metadata<'a>,
        extra: &ParsingError,
    ) -> Report<'a, (&'a str, Range<usize>)> {
        assert!(matches!(extra.extra, Some(ErrorExtra::MissingComma(_))));
        assert!(extra.found.is_none());

        let kind = ReportKind::Warning;

        let span = meta.convert_location(&self.0);
        let loc = (meta.file.name.as_ref(), span);
        let label = Label::new(loc.clone())
            .with_color(Color::Cyan)
            .with_message("Line has no ending with comma".to_string());

        Report::build(kind, loc)
            .with_label(label)
            .with_message(
                "Line has no ending with comma"
                    .fg(Color::Yellow)
                    .to_string(),
            )
            .finish()
    }
}

impl Reportable<ParsingError> for TypedHole {
    fn report<'a>(
        &self,
        meta: &Metadata<'a>,
        extra: &ParsingError,
    ) -> Report<'a, (&'a str, Range<usize>)> {
        assert!(matches!(extra.extra, Some(ErrorExtra::TypedHole(_))));
        assert!(extra.found == Some(Token::Underscore));

        let kind = ReportKind::Warning;

        let span = meta.convert_location(&extra.at);
        let loc = (meta.file.name.as_ref(), span);
        let label = Label::new(loc.clone())
            .with_color(Color::Cyan)
            .with_message(format!("Found {}", (Token::Underscore).fg(Color::Cyan)));

        Report::build(kind, loc)
            .with_label(label)
            .with_message(format!("Found {}.", "TypeHole".fg(Color::Cyan)))
            .finish()
    }
}

impl Reportable<ParsingError> for ParserLexingError {
    fn report<'a>(
        &self,
        meta: &Metadata<'a>,
        extra: &ParsingError,
    ) -> Report<'a, (&'a str, Range<usize>)> {
        assert!(matches!(extra.extra, Some(ErrorExtra::LexingError(_))));
        assert!(extra.found == Some(Token::Err));

        self.0.report(meta, &())
    }
}

impl Reportable<ParsingError> for () {
    fn report<'a>(
        &self,
        meta: &Metadata<'a>,
        extra: &ParsingError,
    ) -> Report<'a, (&'a str, Range<usize>)> {
        assert!(extra.extra.is_none());

        let kind = ReportKind::Error;

        let span = meta.convert_location(&extra.at);
        let loc = (meta.file.name.as_ref(), span);

        let (label, eof) = if let Some(t) = &extra.found {
            (
                Label::new(loc.clone())
                    .with_color(Color::Red)
                    .with_message(format!("Found {}", t.fg(Color::Red)))
                    .into(),
                false,
            )
        } else {
            (None, true)
        };

        let mut message = if eof {
            "Unexpected end of input.".to_string()
        } else {
            "Unexpected token.".to_string()
        };

        if !extra.expected.is_empty() {
            let expected = extra
                .expected
                .iter()
                .map(|p| format!(" {}", format!("\"{p}\"").fg(Color::BrightGreen)));
            message = format!(
                "{message} Expected one of:{}.",
                expected.collect::<String>()
            );
        }

        let mut report = Report::build(kind, loc).with_message(message);
        if let Some(l) = label {
            report = report.with_label(l);
        }

        report.finish()
    }
}

impl Reportable<()> for ParsingError {
    fn report<'a>(&self, meta: &Metadata<'a>, _extra: &()) -> Report<'a, (&'a str, Range<usize>)> {
        match &self.extra {
            Some(ErrorExtra::BadCallChain(e)) => e.report(meta, self),
            Some(ErrorExtra::MissingComma(e)) => e.report(meta, self),
            Some(ErrorExtra::TypedHole(e)) => e.report(meta, self),
            Some(ErrorExtra::LexingError(e)) => e.report(meta, self),
            None => ().report(meta, self),
        }
    }
}

impl Reportable<()> for ElaboratingError {
    fn report<'a>(&self, _meta: &Metadata<'a>, _extra: &()) -> Report<'a, (&'a str, Range<usize>)> {
        unimplemented!()
    }
}
