//! Module implements `ParsingError` struct that
//! is used by parser.
use chumsky::DefaultExpected;
use chumsky::error::Error;
use chumsky::extra::ParserExtra;
use chumsky::input::MapExtra;
use chumsky::input::ValueInput;
use chumsky::label::LabelError;
use chumsky::prelude::*;
use chumsky::util::Maybe;
use chumsky::util::MaybeRef;

use super::Location;
use super::Token;

use crate::error::parsing;
use crate::error::parsing::{ErrorExtra, ExpectedPattern};

pub(super) type ParsingError = parsing::Error;

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
            extra: None,
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

impl<'src, 'b, I, E> From<&mut MapExtra<'src, 'b, I, E>> for ParsingError
where
    I: ValueInput<'src, Span = Location, Token = Token>,
    E: ParserExtra<'src, I>,
{
    fn from(value: &mut MapExtra<'src, 'b, I, E>) -> Self {
        Self {
            found: None,
            expected: vec![],
            at: value.span(),
            extra: None,
        }
    }
}

impl ParsingError {
    #[must_use]
    pub(super) fn bad_call_chain(mut self, loc: Location) -> Self {
        self.extra = Some(ErrorExtra::BadCallChain(loc));
        self
    }

    #[must_use]
    pub(super) fn typed_hole(mut self) -> Self {
        self.extra = Some(ErrorExtra::TypedHole);
        self.found = Some(Token::Underscore);
        self
    }
}
