//! Module containing `Lexer`.
//!
//! Exports:
//!   * `Token` enum for tokens in dbuf files, that implements:
//!     * `Lexer`, so `Token::lexer(&src)` can be used.
//!     * `Locatable` for `Lexer` so `lexer.location()` can be used.
//!
use logos::{Lexer, Logos};
use regex::Regex;
use strum_macros::Display;
use unescape::unescape;

use std::{
    convert::Infallible,
    num::{
        IntErrorKind::{NegOverflow, PosOverflow},
        ParseIntError,
    },
    sync::LazyLock,
};

use super::location::{Locatable, Location, Offset};

use crate::error::lexing::{Error, ErrorData, ErrorKind};

impl ErrorData {
    /// Build `LexingErrorData` based on `lex` state (slice, span).
    fn from_lexer(lex: &mut Lexer<'_, Token>) -> Self {
        Self {
            at: lex.extras.get_offset(lex.span().start),
            current: lex.slice().to_string(),
        }
    }
}

impl Error {
    /// Create `UnknownToken` error for current lexer state.
    ///
    /// Used as default for unknown tokens.
    fn unknown_token(lex: &mut logos::Lexer<'_, Token>) -> Self {
        at_callback(lex);
        Self::from_lexer(lex, ErrorKind::UnknownToken)
    }

    /// Create `kind` error for current lexer state.
    fn from_lexer(lex: &mut logos::Lexer<'_, Token>, kind: ErrorKind) -> Self {
        Error {
            data: ErrorData::from_lexer(lex),
            kind,
        }
    }
}

/// Required for `lex.slice().parse()` construction, since it returns result with
/// Infallible error.
impl From<Infallible> for Error {
    fn from(_value: Infallible) -> Self {
        unreachable!("Since Infallible error couldn't be constructed");
    }
}

/// FIXME.
///
/// Actually there is no need to implement `Default` for
/// `LexingError`, since every unknown token generates
/// error based on callback `bad_token`. But it is
/// required by Logos to error implement `Default`
/// trait, so here it is.
impl Default for Error {
    fn default() -> Self {
        unreachable!();
    }
}

/// Extra data for lexer.
///
/// API
///   * Lexer should call `new_line_at` on every `\n` character in input.
///   * Lexer should call `token_at` on every token in input.
#[derive(Default, Clone)]
pub struct LexingExtra {
    /// Current line number.
    pub line_num: usize,
    /// Start of current line.
    pub line_start: usize,
    /// `Offset` of current token start.
    ///
    /// Need due impossibility to calculate that on
    /// multiline tokens.
    pub token_start: Offset,
}

impl LexingExtra {
    /// Calculates offset of position as if it is on same line.
    fn get_offset(&self, position: usize) -> Offset {
        assert!(position >= self.line_start);

        Offset {
            lines: self.line_num,
            columns: position - self.line_start,
        }
    }

    /// New line handler.
    ///
    /// `position` is a starting position of `\n` symbol.
    fn new_line_at(&mut self, position: usize) {
        self.line_num += 1;
        self.line_start = position + 1;
    }

    /// Token start handler. Saves starting position of token.
    ///
    /// `position` is a starting position of current token.
    fn token_at(&mut self, position: usize) {
        self.token_start = self.get_offset(position);
    }
}

#[derive(Logos, Debug, PartialEq, Clone, Display)]
#[logos(error(Error, Error::unknown_token))]
#[logos(extras = LexingExtra)]
pub enum Token {
    #[token("message", at_callback)]
    Message,
    #[token("enum", at_callback)]
    Enum,

    #[token("true", |lex| at_callback_with(lex, true))]
    #[token("false", |lex| at_callback_with(lex, false))]
    BoolLiteral(bool),
    // Number without u or .
    #[regex(r"[0-9]([a-tv-zA-Z0-9])*", parse_int)]
    IntLiteral(i64),
    // Number without u or ., followed by u, followed by any
    #[regex(r"[0-9]([a-tv-zA-Z0-9])*u[a-zA-Z0-9.]*", parse_uint)]
    UintLiteral(u64),
    // Number without u or ., followed by ., followed by any
    #[regex(r"[0-9]([a-tv-zA-Z0-9])*\.[a-zA-Z0-9.]*", parse_float)]
    FloatLiteral(f64),
    #[regex(r#""([^"\\]|\\.)*""#, parse_string_literal)]
    StringLiteral(String),

    #[regex(r"[A-Z]\w*", parse_uc_identifier)]
    UCIdentifier(String),
    #[regex(r"[a-z]\w*", parse_lc_identifier)]
    LCIdentifier(String),

    #[token("=>", at_callback)]
    Arrow,
    #[token(":", at_callback)]
    Colon,
    #[token(";", at_callback)]
    Semicolon,
    #[token(",", at_callback)]
    Comma,
    #[token(".", at_callback)]
    Dot,
    #[token("(", at_callback)]
    LParen,
    #[token(")", at_callback)]
    RParen,
    #[token("{", at_callback)]
    LBrace,
    #[token("}", at_callback)]
    RBrace,

    #[token("+", at_callback)]
    Plus,
    #[token("-", at_callback)]
    Minus,
    #[token("*", at_callback)]
    Star,
    #[token("/", at_callback)]
    Slash,
    #[token("&", at_callback)]
    Amp,
    #[token("|", at_callback)]
    Pipe,
    #[token("!", at_callback)]
    Bang,

    #[token("_", at_callback)]
    Underscore,

    #[regex(r"\n", newline_callback)]
    Newline,
    #[regex(r"[ \t\r\f]+", at_callback)]
    Space,
    #[regex(r"//[^\n]*", line_comment_callback, allow_greedy = true)]
    LineComment(String),
    #[regex(r"/\*([^*]|\*[^/])*\*/", block_comment_callback)]
    BlockComment(String),

    Err,
}

impl Locatable for Lexer<'_, Token> {
    fn location(&self) -> Location {
        Location::new(
            self.extras.token_start,
            self.extras.get_offset(self.span().end),
        )
        .expect("correct incremental offset behavior")
    }
}

/// Callback that sets start of token.
fn at_callback(lex: &mut Lexer<'_, Token>) {
    lex.extras.token_at(lex.span().start);
}

/// Callback that sets start of token and returns value
fn at_callback_with<T>(lex: &mut Lexer<'_, Token>, value: T) -> T {
    lex.extras.token_at(lex.span().start);
    value
}

/// Callback for `NewLine` token.
///
/// Just update extra.
fn newline_callback(lex: &mut Lexer<'_, Token>) {
    at_callback(lex);
    lex.extras.new_line_at(lex.span().start);
}

/// Callback for `LineComment` token.
///
/// Parses content of comment (currently just saves it to `String`).
fn line_comment_callback(lex: &mut Lexer<'_, Token>) -> Result<String, Error> {
    at_callback(lex);
    lex.slice().parse().map_err(Into::into)
}

/// Callback for `BlockComment` token.
///
/// Updates extra and parses content of comment (currently just saves it to `String`).
///
/// Math inside:
///   * `lex.span().start` is a start of token slice.
///   * `lex.span().start + pos` is a start of `\n` symbol.
fn block_comment_callback(lex: &mut Lexer<'_, Token>) -> Result<String, Error> {
    at_callback(lex);
    let s = lex.slice();
    s.match_indices('\n')
        .for_each(|(pos, _)| lex.extras.new_line_at(lex.span().start + pos));

    s.parse().map_err(Into::into)
}

/// Parser for `IntLiteral` token. Parses i64 and return `Result`.
///
/// Errors
///   * `LexingErrorKind::IntegerOverflow` when integer can't be represented
///     as i64.
///   * `LexingErrorKind::InvalidInteger` when text is not integer.
fn parse_int(lex: &mut Lexer<'_, Token>) -> Result<i64, Error> {
    at_callback(lex);
    lex.slice().parse().map_err(|err: ParseIntError| {
        let kind = match err.kind() {
            PosOverflow | NegOverflow => ErrorKind::IntegerOverflow,
            _ => ErrorKind::InvalidInteger,
        };
        Error::from_lexer(lex, kind)
    })
}

/// Parser for `UintLiteral` token. Removes its suffix 'u', parses u64 and return `Result`.
///
/// Errors
///   * `LexingErrorKind::IntegerOverflow` when integer can't be represented
///     as i64.
///   * `LexingErrorKind::InvalidInteger` when text is not integer.
fn parse_uint(lex: &mut Lexer<'_, Token>) -> Result<u64, Error> {
    at_callback(lex);
    let s = lex.slice();

    s[..s.len() - 1].parse().map_err(|err: ParseIntError| {
        let kind = match err.kind() {
            PosOverflow | NegOverflow => ErrorKind::IntegerOverflow,
            _ => ErrorKind::InvalidInteger,
        };
        Error::from_lexer(lex, kind)
    })
}

/// Parser for `FloatLiteral` token. Parses f64 and return `Result`.
///
/// Errors
///   * `LexingErrorKind::InvalidFloat` text is not float.
fn parse_float(lex: &mut Lexer<'_, Token>) -> Result<f64, Error> {
    at_callback(lex);
    lex.slice()
        .parse()
        .map_err(|_| Error::from_lexer(lex, ErrorKind::InvalidFloat))
}

/// Parser for `StringLiteral` token. Parses string and return `Result`.
///
/// Errors
///   * `LexingErrorKind::InvalidStringLiteral` when literal contains bad escape symbols
///     (such a `\a`, which is heavily outdated).
fn parse_string_literal(lex: &mut Lexer<'_, Token>) -> Result<String, Error> {
    at_callback(lex);
    let s = lex.slice();
    let trimmed = &s[1..s.len() - 1];
    unescape(trimmed).ok_or(Error::from_lexer(lex, ErrorKind::InvalidStringLiteral))
}

/// Parser for `UCIdentifier` token. Parses string and return `Result`.
///
/// Errors
///   * `LexingErrorKind::InvalidUCIdentifier` when uc identifier contains bad characters.
fn parse_uc_identifier(lex: &mut Lexer<'_, Token>) -> Result<String, Error> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^[A-Z][a-zA-Z0-9]*$").expect("correct regular expression"));

    at_callback(lex);
    if RE.is_match(lex.slice()) {
        Ok(lex.slice().into())
    } else {
        Err(Error::from_lexer(lex, ErrorKind::InvalidUCIdentifier))
    }
}

/// Parser for `LCIdentifier` token. Parses string and return `Result`.
///
/// Errors
///   * `LexingErrorKind::InvalidLCIdentifier` when lc identifier contains bad characters.
fn parse_lc_identifier(lex: &mut Lexer<'_, Token>) -> Result<String, Error> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^[a-z][a-zA-Z0-9]*$").expect("correct regular expression"));

    at_callback(lex);
    if RE.is_match(lex.slice()) {
        Ok(lex.slice().into())
    } else {
        Err(Error::from_lexer(lex, ErrorKind::InvalidLCIdentifier))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_same(input: &str, expect: &[Option<Token>]) {
        let tokens: Vec<_> = Token::lexer(input).collect();

        assert!(
            tokens.len() == expect.len(),
            "[input='{input}'] Expected {} tokens in answer, got {}",
            expect.len(),
            tokens.len()
        );

        let lex = Token::lexer(input);
        for (ans, expect) in lex.into_iter().zip(expect.iter()) {
            match (ans, expect) {
                (Ok(t), None) => {
                    panic!("[input='{input}'] Expected error token, got '{t:?}'");
                }
                (Ok(t1), Some(t2)) => {
                    assert!(
                        t1 == *t2,
                        "[input='{input}'] Expected token '{t2:?}', got token '{t1:?}'"
                    );
                }
                (Err(_), None) => {}
                (Err(err), Some(t)) => {
                    panic!("[input='{input}'] Expected token '{t:?}', got error: '{err}'");
                }
            }
        }
    }

    #[test]
    fn test_number_correct() {
        test_same("123", &[Some(Token::IntLiteral(123))]);
        test_same("123u", &[Some(Token::UintLiteral(123))]);
        test_same("123.32", &[Some(Token::FloatLiteral(123.32))]);

        test_same(
            "123+32",
            &[
                Some(Token::IntLiteral(123)),
                Some(Token::Plus),
                Some(Token::IntLiteral(32)),
            ],
        );
        test_same(
            "123u+32u",
            &[
                Some(Token::UintLiteral(123)),
                Some(Token::Plus),
                Some(Token::UintLiteral(32)),
            ],
        );
        test_same(
            "123.32+32.32",
            &[
                Some(Token::FloatLiteral(123.32)),
                Some(Token::Plus),
                Some(Token::FloatLiteral(32.32)),
            ],
        );
    }

    #[test]
    fn test_string_correct() {
        test_same("\"aba\"", &[Some(Token::StringLiteral("aba".into()))]);
        test_same(
            "\"\\\"aba\\\"\"",
            &[Some(Token::StringLiteral("\"aba\"".into()))],
        );
        test_same(
            "\"aba\\naba\"",
            &[Some(Token::StringLiteral("aba\naba".into()))],
        );
    }

    #[test]
    fn test_comment_correct() {
        test_same(
            "// comment line",
            &[Some(Token::LineComment("// comment line".into()))],
        );
        test_same(
            "// comment line\n// other comment",
            &[
                Some(Token::LineComment("// comment line".into())),
                Some(Token::Newline),
                Some(Token::LineComment("// other comment".into())),
            ],
        );
        test_same("// / // a", &[Some(Token::LineComment("// / // a".into()))]);

        test_same(
            "a /* comment */ b",
            &[
                Some(Token::LCIdentifier("a".into())),
                Some(Token::Space),
                Some(Token::BlockComment("/* comment */".into())),
                Some(Token::Space),
                Some(Token::LCIdentifier("b".into())),
            ],
        );
        test_same(
            "/* comment */",
            &[Some(Token::BlockComment("/* comment */".into()))],
        );
        test_same(
            "/* comment */\n",
            &[
                Some(Token::BlockComment("/* comment */".into())),
                Some(Token::Newline),
            ],
        );
        test_same(
            "/* * / */",
            &[Some(Token::BlockComment("/* * / */".into()))],
        );
        test_same(
            "/*\n * multi line\n * comment\n*/ /* other *//* other2 */",
            &[
                Some(Token::BlockComment(
                    "/*\n * multi line\n * comment\n*/".into(),
                )),
                Some(Token::Space),
                Some(Token::BlockComment("/* other */".into())),
                Some(Token::BlockComment("/* other2 */".into())),
            ],
        );
    }
}
