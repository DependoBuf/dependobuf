use std::ops::{Deref, DerefMut, Range};

use crate::{ast::parsed::location::Offset, parser::lexer::Token};

use logos::{Lexer, Logos};

pub struct LocatedIter<Lex> {
    lexer: Lex,
}

pub trait LocatedIterTrait {
    fn located(self) -> LocatedIter<Self>
    where
        Self: Sized;
}

impl<'source, Token: Logos<'source>> LocatedIterTrait for Lexer<'source, Token> {
    fn located(self) -> LocatedIter<Lexer<'source, Token>> {
        LocatedIter { lexer: self }
    }
}

impl<'source> Iterator for LocatedIter<Lexer<'source, Token>> {
    type Item = (
        Result<Token, <Token as Logos<'source>>::Error>,
        Range<Offset>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|token| {
            assert!(self.lexer.extras.1 <= self.lexer.span().start);
            let start = Offset {
                lines: self.lexer.extras.0,
                columns: self.lexer.span().start - self.lexer.extras.1,
            };
            let end = Offset {
                lines: self.lexer.extras.0,
                columns: self.lexer.span().end - self.lexer.extras.1,
            };
            (token, start..end)
        })
    }
}

impl<'source> Deref for LocatedIter<Lexer<'source, Token>> {
    type Target = Lexer<'source, Token>;

    fn deref(&self) -> &Lexer<'source, Token> {
        &self.lexer
    }
}

impl<'source> DerefMut for LocatedIter<Lexer<'source, Token>> {
    fn deref_mut(&mut self) -> &mut Lexer<'source, Token> {
        &mut self.lexer
    }
}
