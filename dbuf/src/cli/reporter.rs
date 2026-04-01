use std::ops::Range;

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

use dbuf_core::cst::Token;
use dbuf_core::location::Location;
use dbuf_core::location::Offset;

use dbuf_core::error::parsing::ErrorExtra;
use dbuf_core::error::{Error, elaborating, lexing, parsing};

use super::file::File;

pub struct Reporter<'a> {
    file: &'a File,
    newlines: Vec<usize>,
    reports: Vec<Report<'a, (&'a str, Range<usize>)>>,
}

impl<'a> Reporter<'a> {
    pub fn new(file: &'a File) -> Self {
        let mut newlines = vec![0];
        for (i, ch) in file.content.chars().enumerate() {
            if ch == '\n' {
                newlines.push(i + 1);
            }
        }

        Self {
            file,
            newlines,
            reports: vec![],
        }
    }

    pub fn print(self) {
        for report in self.reports {
            report
                .eprint((self.file.name.as_ref(), Source::from(&self.file.content)))
                .unwrap();
        }
    }

    pub fn report(&mut self, err: Error) {
        match err {
            Error::LexingError(error) => self.lexing_error(error),
            Error::ParsingError(error) => self.parsing_error(&error),
            Error::ElaboratingError(error) => self.elaborating_error(error),
        }
    }

    fn lexing_error(&mut self, err: lexing::Error) {
        let (kind, c) = if err.kind == lexing::ErrorKind::UnknownToken {
            (ReportKind::Warning, Color::Yellow)
        } else {
            (ReportKind::Error, Color::Red)
        };

        let location_start = self.convert_offset(err.data.at);
        let location_end = location_start + err.data.current.len();
        let span = location_start..location_end;
        let loc = (self.file.name.as_ref(), span);

        let report = Report::build(kind, loc.clone())
            .with_message(err.kind)
            .with_label(Label::new(loc).with_color(c))
            .finish();
        self.reports.push(report);
    }
    fn parsing_error(&mut self, err: &parsing::Error) {
        match err.extra {
            Some(ErrorExtra::BadCallChain(l)) => self.parsing_bad_call_chain(err, l),
            Some(ErrorExtra::TypedHole) => self.parsing_type_hole(err),
            Some(ErrorExtra::MissingComma(l)) => self.parsing_missing_comma(err, l),
            None => self.parsing_regular_error(err),
        }
    }

    /// Reports parsing bad call chain error.
    ///
    /// Arguments:
    /// * `err`: error to report.
    /// * `location`: definition of whole call chain
    fn parsing_bad_call_chain(&mut self, err: &parsing::Error, location: Location<Offset>) {
        assert!(matches!(err.extra, Some(ErrorExtra::BadCallChain(_))));
        assert!(err.found == Some(Token::Dot));

        let kind = ReportKind::Warning;

        let span = self.convert_location(&err.at);
        let loc1 = (self.file.name.as_ref(), span);
        let label1 = Label::new(loc1.clone())
            .with_color(Color::Yellow)
            .with_message(format!("Found {}", (Token::Dot).fg(Color::Yellow)));

        let span = self.convert_location(&location);
        let loc2 = (self.file.name.as_ref(), span);
        let label2 = Label::new(loc2)
            .with_color(Color::Cyan)
            .with_message("Unfinished call chain");

        let report = Report::build(kind, loc1)
            .with_label(label1)
            .with_label(label2)
            .with_message("Call chain not finished".to_string())
            .finish();

        self.reports.push(report);
    }

    /// Reports parsing missing comma error.
    fn parsing_missing_comma(&mut self, err: &parsing::Error, location: Location<Offset>) {
        assert!(matches!(err.extra, Some(ErrorExtra::MissingComma(_))));
        assert!(err.found.is_none());

        let kind = ReportKind::Warning;

        let span = self.convert_location(&location);
        let loc = (self.file.name.as_ref(), span);
        let label = Label::new(loc.clone())
            .with_color(Color::Cyan)
            .with_message("Line has no ending with comma".to_string());

        let report = Report::build(kind, loc)
            .with_label(label)
            .with_message(
                "Line has no ending with comma"
                    .fg(Color::Yellow)
                    .to_string(),
            )
            .finish();

        self.reports.push(report);
    }

    /// Reports parsing type hole error.
    fn parsing_type_hole(&mut self, err: &parsing::Error) {
        assert!(err.extra == Some(ErrorExtra::TypedHole));
        assert!(err.found == Some(Token::Underscore));

        let kind = ReportKind::Warning;

        let span = self.convert_location(&err.at);
        let loc = (self.file.name.as_ref(), span);
        let label = Label::new(loc.clone())
            .with_color(Color::Cyan)
            .with_message(format!("Found {}", (Token::Underscore).fg(Color::Cyan)));

        let report = Report::build(kind, loc)
            .with_label(label)
            .with_message(format!("Found {}.", "TypeHole".fg(Color::Cyan)))
            .finish();

        self.reports.push(report);
    }

    /// Reports regular parsing error.
    fn parsing_regular_error(&mut self, err: &parsing::Error) {
        assert!(err.extra.is_none());

        let kind = ReportKind::Error;

        let span = self.convert_location(&err.at);
        let loc = (self.file.name.as_ref(), span);

        let (label, eof) = if let Some(t) = &err.found {
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

        if !err.expected.is_empty() {
            let expected = err
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

        self.reports.push(report.finish());
    }

    fn elaborating_error(&mut self, _err: elaborating::Error) {
        unimplemented!()
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
