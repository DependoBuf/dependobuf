use std::ops::Range;

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

use dbuf_core::ast::parsed::location::Offset;
use dbuf_core::cst::Location;
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
            Error::ParsingError(error) => self.parsing_error(error),
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
    fn parsing_error(&mut self, err: parsing::Error) {
        let (kind, c) = match err.extra {
            Some(ErrorExtra::BadCallChain(_)) => (ReportKind::Warning, Color::Yellow),
            Some(ErrorExtra::TypedHole) => (ReportKind::Warning, Color::Cyan),
            None => (ReportKind::Error, Color::Red),
        };

        let span = self.convert_location(&err.at);
        let loc = (self.file.name.as_ref(), span);

        let mut label = Label::new(loc.clone()).with_color(c);
        let mut eof = true;
        if let Some(t) = err.found {
            eof = false;
            label = label.with_message(format!("Found {}", t.fg(c)));
        }

        let mut report = Report::build(kind, loc).with_label(label);

        let message = match err.extra {
            Some(ErrorExtra::BadCallChain(loc)) => {
                let span = self.convert_location(&loc);
                let span_correct = span.start..(span.end + 1);
                let loc = (self.file.name.as_ref(), span_correct);
                let label = Label::new(loc)
                    .with_color(Color::Cyan)
                    .with_message("Unfinished call chain");

                report = report.with_label(label);
                "Call chain not finished.".to_string()
            }
            Some(ErrorExtra::TypedHole) => {
                format!("Found {}.", "TypeHole".fg(Color::Cyan))
            }
            None => {
                let mut ans = if eof {
                    "Unexpected end of input.".to_string()
                } else {
                    "Unexpected token.".to_string()
                };

                if !err.expected.is_empty() {
                    ans += " Expected one of:";
                    for p in err.expected {
                        let cur = format!("\"{p}\"");
                        ans = format!("{ans} {}", cur.fg(Color::BrightGreen));
                    }
                }
                ans
            }
        };

        let report = report.with_message(message).finish();

        self.reports.push(report);
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

    fn convert_location(&self, loc: &Location) -> Range<usize> {
        let location_start = self.convert_offset(loc.start());
        let location_end = self.convert_offset(loc.end());
        location_start..location_end
    }
}
