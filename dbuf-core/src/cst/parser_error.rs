//! Module exports `ParsingError` struct that
//! is used by parser.
use chumsky::DefaultExpected;
use chumsky::error::Error;
use chumsky::input::ValueInput;
use chumsky::label::LabelError;
use chumsky::prelude::*;
use chumsky::util::Maybe;
use chumsky::util::MaybeRef;

use super::Location;
use super::Token;

/// Possible expected tokens for parser.
#[derive(Clone, Debug, PartialEq)]
enum ExpectedPattern {
    Token(Token),
    Label(&'static str),
    Any,
    SomethingElse,
    EndOfInput,
}

impl<'src> From<DefaultExpected<'src, Token>> for ExpectedPattern {
    fn from(value: DefaultExpected<'src, Token>) -> Self {
        match value {
            DefaultExpected::Token(Maybe::Ref(r)) => Self::Token(r.to_owned()),
            DefaultExpected::Token(Maybe::Val(v)) => Self::Token(v),
            DefaultExpected::Any => Self::Any,
            DefaultExpected::SomethingElse => Self::SomethingElse,
            DefaultExpected::EndOfInput => Self::EndOfInput,
            _ => panic!("unknown expected"),
        }
    }
}

impl From<&'static str> for ExpectedPattern {
    fn from(value: &'static str) -> Self {
        Self::Label(value)
    }
}

/// Parsing error, that implements `LabelError`
/// and `Error` required by `chumsky`.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ParsingError {
    found: Option<Token>,
    expected: Vec<ExpectedPattern>,
    at: Location,
}

impl<'src, I, L> LabelError<'src, I, L> for ParsingError
where
    I: Input<'src, Span = Location, Token = Token>,
    L: Into<ExpectedPattern>,
{
    fn expected_found<E: IntoIterator<Item = L>>(
        expected: E,
        found: Option<MaybeRef<'src, Token>>,
        span: Location,
    ) -> Self {
        let found = found.map(|maybe| match maybe {
            Maybe::Ref(r) => r.to_owned(),
            Maybe::Val(v) => v,
        });
        ParsingError {
            found,
            expected: expected.into_iter().map(Into::into).collect(),
            at: span,
        }
    }

    fn label_with(&mut self, label: L) {
        self.expected.clear();
        self.expected.push(label.into());
    }
}

impl<'src, I> Error<'src, I> for ParsingError
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    fn merge(mut self, other: Self) -> Self {
        other.expected.into_iter().for_each(|e| {
            (!self.expected.contains(&e)).then(|| self.expected.push(e));
        });

        self
    }
}
