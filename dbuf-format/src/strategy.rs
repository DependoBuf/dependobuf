//! Module contains strategy for pretty-printing different parts of CST
//!

use std::borrow::Cow;
use std::cmp::max;
use std::ops::BitAnd;

use super::utils::{Event, PrettyStrategy};

use dbuf_core::cst::{Token, TreeKind};
use dbuf_core::location::{Location, Offset};

use pretty::{DocAllocator, DocBuilder};

/// Returns correspond token representation.
/// Requires Location since spaces are not gathered.
fn token_to_string<'a>(t: &'a Token, l: &'a Location<Offset>) -> Cow<'a, str> {
    match t {
        Token::Message => "message".into(),
        Token::Enum => "enum".into(),
        Token::BoolLiteral(l) => l.to_string().into(),
        Token::IntLiteral(l) => l.to_string().into(),
        Token::UintLiteral(l) => format!("{l}u").into(),
        Token::StringLiteral(l) => format!("\"{l}\"").into(),
        Token::UCIdentifier(l) => l.into(),
        Token::LCIdentifier(l) => l.into(),
        Token::Arrow => "=>".into(),
        Token::Colon => ":".into(),
        Token::Semicolon => ";".into(),
        Token::Comma => ",".into(),
        Token::Dot => ".".into(),
        Token::LParen => "(".into(),
        Token::RParen => ")".into(),
        Token::LBrace => "{".into(),
        Token::RBrace => "}".into(),
        Token::Plus => "+".into(),
        Token::Minus => "-".into(),
        Token::Star => "*".into(),
        Token::Amp => "&".into(),
        Token::Pipe => "|".into(),
        Token::Bang => "!".into(),
        Token::Underscore => "_".into(),
        Token::NewLine => "\n".into(),
        Token::Space => " ".repeat(l.length.columns).into(),
        Token::LineComment(c) => c.into(),
        Token::BlockComment(c) => c.into(),
        Token::Err(c) => c.into(),
    }
}

/// Policy for printing spaces.
#[derive(Clone, Copy)]
enum SpacePolicy {
    /// Forces to not print space.
    NoSpace,
    /// Print space if next token agree.
    MaybeSpace,
}

impl BitAnd for SpacePolicy {
    type Output = SpacePolicy;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (SpacePolicy::NoSpace, SpacePolicy::NoSpace) => SpacePolicy::NoSpace,
            (SpacePolicy::NoSpace, SpacePolicy::MaybeSpace) => SpacePolicy::NoSpace,
            (SpacePolicy::MaybeSpace, SpacePolicy::NoSpace) => SpacePolicy::NoSpace,
            (SpacePolicy::MaybeSpace, SpacePolicy::MaybeSpace) => SpacePolicy::MaybeSpace,
        }
    }
}

/// Policy for printing new lines.
/// Order by priority from least to most.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum LinePolicy {
    /// Print new line if next token agree.
    MaybeLine,
    /// If user had new line between tokens then
    /// forces new line. If no, then no new line.
    UserBased,
    /// Two new lines.
    TwoLine,
    /// Forces to not print new line.
    NoLine,
}

impl BitAnd for LinePolicy {
    type Output = LinePolicy;

    fn bitand(self, rhs: Self) -> Self::Output {
        max(self, rhs)
    }
}

/// Policy for before/after token spaces.
#[derive(Clone, Copy)]
struct TokenPolicy {
    /// space policy.
    space: SpacePolicy,
    /// line policy.
    line: LinePolicy,
}

impl BitAnd for TokenPolicy {
    type Output = TokenPolicy;

    fn bitand(self, rhs: Self) -> Self::Output {
        TokenPolicy {
            space: self.space & rhs.space,
            line: self.line & rhs.line,
        }
    }
}

/// Configuration for pretty printing strategy.
#[derive(Clone, Copy)]
pub struct StrategyConfig {
    /// size of tab in spaces.
    pub tab_size: usize,
}

/// Strategy for pretty printing code.
pub struct Strategy<'a> {
    /// increase in indent on new tab.
    tab_size: usize,
    /// current indent.
    indent: usize,
    /// last token
    last_token: Option<&'a Token>,
    /// token policy for last token.
    last_policy: TokenPolicy,
    /// represent if user had new line before token
    user_newline: bool,
    /// scopes for current tree node.
    scope: TreeKind,
}

impl<'a> Strategy<'a> {
    /// create new strategy based on config.
    pub fn new(conf: StrategyConfig) -> Strategy<'a> {
        let mut scopes = Vec::with_capacity(10);
        scopes.push(TreeKind::File);
        Strategy {
            tab_size: conf.tab_size,
            indent: 0,
            last_token: None,
            last_policy: TokenPolicy {
                space: SpacePolicy::NoSpace,
                line: LinePolicy::NoLine,
            },
            user_newline: false,
            scope: TreeKind::File,
        }
    }

    /// increase indent by one tab.
    fn plus_tab(&mut self) {
        self.indent += self.tab_size;
    }

    /// decrease indent by one tab.
    fn minus_tab(&mut self) {
        if self.indent < self.tab_size {
            self.indent = 0;
        } else {
            self.indent -= self.tab_size;
        }
    }

    /// changes tab size before applying token.
    fn change_tab_size_before(&mut self, t: &Token) {
        if matches!(
            self.scope,
            TreeKind::ConstructedPattern | TreeKind::ConstructedValue
        ) {
            return;
        }

        if matches!(t, Token::RBrace) {
            self.minus_tab();
        }
    }

    /// changes tab size after applying token.
    fn change_tab_size_after(&mut self, t: &Token) {
        if matches!(
            self.scope,
            TreeKind::ConstructedPattern | TreeKind::ConstructedValue
        ) {
            return;
        }

        if matches!(t, Token::LBrace) {
            self.plus_tab();
        }
    }

    /// returns token policy that should be applied before token.
    fn before_token_policy(&self, t: &Token) -> TokenPolicy {
        if matches!((self.last_token, t), (Some(Token::LBrace), Token::RBrace)) {
            return TokenPolicy {
                space: SpacePolicy::NoSpace,
                line: LinePolicy::NoLine,
            };
        }

        let ctr_scope = matches!(
            self.scope,
            TreeKind::ConstructedPattern | TreeKind::ConstructedValue
        );

        let space = match t {
            Token::LBrace if ctr_scope => SpacePolicy::NoSpace,
            Token::RBrace if ctr_scope => SpacePolicy::NoSpace,
            Token::RParen => SpacePolicy::NoSpace,
            Token::Semicolon => SpacePolicy::NoSpace,
            Token::Colon => SpacePolicy::NoSpace,
            Token::Comma => SpacePolicy::NoSpace,
            Token::Dot => SpacePolicy::NoSpace,
            _ => SpacePolicy::MaybeSpace,
        };

        TokenPolicy {
            space,
            line: LinePolicy::MaybeLine,
        }
    }

    /// returns token policy that should be applied after token.
    fn after_token_policy(&self, t: &Token) -> TokenPolicy {
        let ctr_scope = matches!(
            self.scope,
            TreeKind::ConstructedPattern | TreeKind::ConstructedValue
        );

        let space = match t {
            Token::LBrace if ctr_scope => SpacePolicy::NoSpace,
            Token::LParen => SpacePolicy::NoSpace,
            Token::Dot => SpacePolicy::NoSpace,
            _ => SpacePolicy::MaybeSpace,
        };

        let line = match t {
            Token::LineComment(_) => LinePolicy::MaybeLine,
            Token::Semicolon => LinePolicy::MaybeLine,
            Token::LBrace if !ctr_scope => LinePolicy::MaybeLine,
            Token::RBrace if !ctr_scope => LinePolicy::MaybeLine,
            _ => LinePolicy::UserBased,
        };

        TokenPolicy { space, line }
    }

    /// returns `DocBuilder`, representing new line + tab.
    fn alloc_line<D>(&self, allocator: &'a D) -> DocBuilder<'a, D>
    where
        D: DocAllocator<'a>,
        D::Doc: Clone,
    {
        let spaces = " ".repeat(self.indent);
        allocator.hardline().append(allocator.text(spaces))
    }

    /// applies ws policy and returns result in `DocBuilder`.
    fn apply_policy<D>(&self, policy: TokenPolicy, allocator: &'a D) -> DocBuilder<'a, D>
    where
        D: DocAllocator<'a>,
        D::Doc: Clone,
    {
        match policy.line {
            LinePolicy::MaybeLine => return self.alloc_line(allocator),
            LinePolicy::UserBased if self.user_newline => return self.alloc_line(allocator),
            LinePolicy::UserBased => (),
            LinePolicy::TwoLine => {
                return self
                    .alloc_line(allocator)
                    .append(self.alloc_line(allocator));
            }
            LinePolicy::NoLine => (),
        }

        match policy.space {
            SpacePolicy::NoSpace => allocator.nil(),
            SpacePolicy::MaybeSpace => allocator.text(" "),
        }
    }
}

impl<'a, D> PrettyStrategy<'a, D> for Strategy<'a> {
    fn next(&mut self, event: Event<'a>, allocator: &'a D) -> DocBuilder<'a, D>
    where
        D: DocAllocator<'a>,
        D::Doc: Clone,
    {
        match event {
            Event::NextToken(token, location) => {
                if matches!(token, Token::NewLine) {
                    self.user_newline = true;
                    return allocator.nil();
                }
                if matches!(token, Token::Space) {
                    return allocator.nil();
                }

                self.change_tab_size_before(token);

                let before = self.before_token_policy(token);
                let cur = self.last_policy & before;
                let doc = self.apply_policy(cur, allocator);

                let doc = doc.append(allocator.text(token_to_string(token, location)));

                self.change_tab_size_after(token);

                self.last_policy = self.after_token_policy(token);
                self.last_token = token.into();
                self.user_newline = false;

                doc
            }
            Event::NewScope(tree_kind) => {
                self.scope = tree_kind.clone();
                allocator.nil()
            }
            Event::ExitScope(tree_kind) => {
                self.scope = tree_kind.clone();

                if matches!(tree_kind, TreeKind::File) {
                    self.last_policy.line = LinePolicy::TwoLine;
                }

                allocator.nil()
            }
        }
    }
}
