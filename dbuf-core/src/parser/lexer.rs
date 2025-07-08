use logos::{Lexer, Logos, Skip};
use unescape::unescape;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(extras = (usize, usize))]
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

    #[regex(r"\n", newline_callback)]
    Newline,

    #[regex(r"[ \t\r\f]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/", logos::skip)]
    Err,
}

fn newline_callback(lex: &mut Lexer<Token>) -> Skip {
    lex.extras.0 += 1;
    lex.extras.1 = lex.span().end;
    Skip
}

fn parse_string(s: &str) -> Option<String> {
    let trimmed = &s[1..s.len() - 1];
    unescape(trimmed)
}

fn parse_uint(s: &str) -> Option<u64> {
    s[..s.len() - 1].parse().ok() // remove 'u' suffix
}
