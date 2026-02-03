//! Module that exports `LocatedLexer` - a lexer which iterator
//! returns `(Token, Location)`
//!
use super::location::{Locatable, Location};

/// Lexer abstraction that returns its tokens
/// with their locations.
#[derive(Clone)]
pub struct LocatedLexer<Lex> {
    lex: Lex,
}

impl<Lex: Locatable + Iterator> LocatedLexer<Lex> {
    /// Create `LocatedLexer` from `Lexer`
    pub fn from_lexer(lex: Lex) -> Self {
        Self { lex }
    }
}

impl<Lex: Locatable + Iterator> Iterator for LocatedLexer<Lex> {
    type Item = (<Lex as Iterator>::Item, Location);

    fn next(&mut self) -> Option<Self::Item> {
        self.lex.next().map(|token| (token, self.lex.location()))
    }
}
