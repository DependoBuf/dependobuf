use logos::{Lexer, Logos, Skip};
use unescape::unescape;

use strum::EnumMessage;
use strum_macros::EnumMessage;
use thiserror::Error;

use std::{
    convert::Infallible,
    num::{
        IntErrorKind::{NegOverflow, PosOverflow},
        ParseIntError,
    },
};

use crate::ast::parsed::location::Offset;

/// Common lexing error data.
#[derive(Debug, Clone, PartialEq)]
pub struct LexingErrorData {
    /// Starting position of token, raised error.
    pub at: Offset,
    /// String representation of token, raised error.
    pub current: String,
}

impl LexingErrorData {
    /// Build `LexingErrorData` based on `lex` state (slice, span).
    fn from_lexer(lex: &mut logos::Lexer<'_, Token>) -> Self {
        Self {
            at: lex.extras.get_offset(lex.span().start),
            current: lex.slice().to_string(),
        }
    }
}

/// All lexing error kinds.
///
/// Every variant should have doc comment, explaining it.
#[derive(Debug, Clone, PartialEq, EnumMessage)]
pub enum LexingErrorKind {
    /// Integer is too huge.
    IntegerOverflow,
    /// Integer is incorrect.
    InvalidInteger,
    /// Float is incorrect.
    InvalidFloat,
    /// String literal is incorrect.
    InvalidStringLiteral,
    /// Unknown token.
    UnknownToken,
}

/// General lexing errors structure.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("[ln {}, ch {}]: Token '{}' raised error: {}", 
    {data.at.lines}, {data.at.columns}, {&data.current},
    {kind.get_documentation().expect("every enum variant has documentation")})]
pub struct LexingError {
    /// Additional data to error.
    pub data: LexingErrorData,
    /// Kind of error.
    pub kind: LexingErrorKind,
}

impl LexingError {
    /// Create `UnknownToken` error for current lexer state.
    ///
    /// Used as default for unknown tokens.
    fn unknown_token(lex: &mut logos::Lexer<'_, Token>) -> Self {
        Self::from_lexer(lex, LexingErrorKind::UnknownToken)
    }

    /// Create `kind` error for current lexer state.
    fn from_lexer(lex: &mut logos::Lexer<'_, Token>, kind: LexingErrorKind) -> Self {
        LexingError {
            data: LexingErrorData::from_lexer(lex),
            kind,
        }
    }
}

/// Required for `lex.slice().parse()` construction, since it returns result with
/// Infallible error.
impl From<Infallible> for LexingError {
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
impl Default for LexingError {
    fn default() -> Self {
        unreachable!();
    }
}

/// Extra data for lexer.
#[derive(Default)]
pub struct LexingExtra {
    /// Current line number.
    line_num: usize,
    /// Start of current line.
    line_start: usize,
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
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error(LexingError, LexingError::unknown_token))]
#[logos(extras = LexingExtra)]
pub enum Token {
    #[token("message")]
    Message,
    #[token("enum")]
    Enum,

    #[token("true", |_| true)]
    #[token("false", |_| false)]
    BoolLiteral(bool),
    #[regex(r"[0-9]+", parse_int)]
    IntLiteral(i64),
    #[regex(r"[0-9]+u", parse_uint)]
    UintLiteral(u64),
    #[regex(r"[0-9]+\.[0-9]+", parse_float)]
    FloatLiteral(f64),
    #[regex(r#""([^"\\]|\\.)*""#, parse_string_literal)]
    StringLiteral(String),

    #[regex(r"[A-Z][A-Za-z0-9_]*", |lex| lex.slice().parse())]
    UCIdentifier(String),
    #[regex(r"[a-z][A-Za-z0-9_]*", |lex| lex.slice().parse())]
    LCIdentifier(String),

    #[token("=>")]
    Arrow,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("&")]
    Amp,
    #[token("|")]
    Pipe,
    #[token("!")]
    Bang,

    #[regex(r"\n", newline_callback)]
    Newline,

    #[regex(r"//.*?\n", line_comment_callback)]
    LineComment(String),
    /// FIXME.
    ///
    /// When Issue <https://github.com/maciejhirsz/logos/issues/532> is resolved
    /// can be simplified to "(?m)\/\*.*?\*\/".
    #[regex(r"/\*([^*]|\*[^/])*\*/", block_comment_callback)]
    BlockComment(String),

    #[regex(r"[ \t\r\f]+", logos::skip)]
    Err,
}

/// Callback for `NewLine` token.
///
/// Just update extra.
fn newline_callback(lex: &mut Lexer<'_, Token>) -> Skip {
    lex.extras.new_line_at(lex.span().start);
    Skip
}

/// Callback for `LineComment` token.
///
/// Updates extra and parses content of comment (currently just saves it to `String`).
fn line_comment_callback(lex: &mut Lexer<'_, Token>) -> Result<String, LexingError> {
    lex.extras.new_line_at(lex.span().start);
    lex.slice().parse().map_err(Into::into)
}

/// Callback for `BlockComment` token.
///
/// Updates extra and parses content of comment (currently just saves it to `String`).
///
/// Math inside:
///   * `lex.span().start` is a start of token slice.
///   * `lex.span().start + pos` is a start of `\n` symbol.
fn block_comment_callback(lex: &mut Lexer<'_, Token>) -> Result<String, LexingError> {
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
///   * `LexingErrorKind::InvalidInteger` in other cases (probably never).
fn parse_int(lex: &mut Lexer<'_, Token>) -> Result<i64, LexingError> {
    lex.slice().parse().map_err(|err: ParseIntError| {
        let kind = match err.kind() {
            PosOverflow | NegOverflow => LexingErrorKind::IntegerOverflow,
            _ => LexingErrorKind::InvalidInteger,
        };
        LexingError::from_lexer(lex, kind)
    })
}

/// Parser for `UintLiteral` token. Removes its suffix 'u', parses u64 and return `Result`.
///
/// Errors
///   * `LexingErrorKind::IntegerOverflow` when integer can't be represented
///     as i64.
///   * `LexingErrorKind::InvalidInteger` in other cases (probably never).
fn parse_uint(lex: &mut Lexer<'_, Token>) -> Result<u64, LexingError> {
    let s = lex.slice();

    s[..s.len() - 1].parse().map_err(|err: ParseIntError| {
        let kind = match err.kind() {
            PosOverflow | NegOverflow => LexingErrorKind::IntegerOverflow,
            _ => LexingErrorKind::InvalidInteger,
        };
        LexingError::from_lexer(lex, kind)
    })
}

/// Parser for `FloatLiteral` token. Parses f64 and return `Result`.
///
/// Errors
///   * `LexingErrorKind::InvalidFloat` when float cannot be parsed. (probably never).
fn parse_float(lex: &mut Lexer<'_, Token>) -> Result<f64, LexingError> {
    lex.slice()
        .parse()
        .map_err(|_| LexingError::from_lexer(lex, LexingErrorKind::InvalidFloat))
}

/// Parser for `StringLiteral` token. Parses string and return `Result`.
///
/// Errors
///   * `LexingErrorKind::InvalidStringLiteral` when literal contains bad escape symbols
///     (such a `\a`, which is heavily outdated).
fn parse_string_literal(lex: &mut Lexer<'_, Token>) -> Result<String, LexingError> {
    let s = lex.slice();
    let trimmed = &s[1..s.len() - 1];
    unescape(trimmed).ok_or(LexingError::from_lexer(
        lex,
        LexingErrorKind::InvalidStringLiteral,
    ))
}
