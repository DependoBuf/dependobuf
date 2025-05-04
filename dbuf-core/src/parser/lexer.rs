use chumsky::span::SimpleSpan;
use logos::Logos;
use unescape::unescape;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[token("message")]
    Message,
    #[token("enum")]
    Enum,

    #[token("true", |_| true)]
    #[token("false", |_| false)]
    BoolLiteral(bool),
    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    IntLiteral(i64),
    #[regex(r"[0-9]+u", |lex| parse_uint(lex.slice()))]
    UintLiteral(u64),
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse().ok())]
    FloatLiteral(f64),
    #[regex(r#""([^"\\]|\\.)*""#, |lex| parse_string(lex.slice()))]
    StringLiteral(String),

    #[regex(r"[A-Z][A-Za-z0-9_]*", |lex| lex.slice().parse().ok())]
    UCIdentifier(String),
    #[regex(r"[a-z][A-Za-z0-9_]*", |lex| lex.slice().parse().ok())]
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

    #[regex(r"[ \t\r\n\f]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/", logos::skip)]
    Error,
}

fn parse_string(s: &str) -> Option<String> {
    let trimmed = &s[1..s.len() - 1];
    unescape(trimmed)
}

fn parse_uint(s: &str) -> Option<u64> {
    s[..s.len() - 1].parse().ok() // remove 'u' suffix
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct Span {
    start: usize,
    end: usize,
}

impl<C> From<SimpleSpan<usize, C>> for Span {
    fn from(value: SimpleSpan<usize, C>) -> Self {
        Span {
            start: value.start,
            end: value.end,
        }
    }
}
