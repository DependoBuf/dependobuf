//! Module exports:
//!   * `file_parser` function, which builds `chumsky::Parser` for dbuf language.
use chumsky::extra::*;
use chumsky::input::ValueInput;
use chumsky::inspector::Inspector;
use chumsky::pratt::*;
use chumsky::prelude::*;

use super::{Child, Token, Tree, TreeKind};
use crate::cst::parser_utils::ChildFlatten;
use crate::location::Location;
use crate::location::Offset;

use super::parser_error::ParsingError;
use super::parser_utils::{MapChild, MapToken, MapTree};

/// State of parsing used for emitting correct locations of errors.
#[derive(Clone, Default)]
pub struct ParsingState {
    call_chain_start: Option<Location<Offset>>,
    definition_start: Option<Location<Offset>>,
}

impl<'src, I> Inspector<'src, I> for ParsingState
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    type Checkpoint = ParsingState;

    fn on_token(&mut self, _token: &Token) {}

    fn on_save<'parse>(
        &self,
        _cursor: &chumsky::input::Cursor<'src, 'parse, I>,
    ) -> Self::Checkpoint {
        self.clone()
    }

    fn on_rewind<'parse>(
        &mut self,
        marker: &chumsky::input::Checkpoint<'src, 'parse, I, Self::Checkpoint>,
    ) {
        *self = marker.inspector().clone();
    }
}

/// type for extra of parser
type ExtraData = Full<ParsingError, ParsingState, ()>;

/// Parses one dbuf file.
///
/// Pattern:
/// ```dbuf
/// (<message> | <enum> | /* comment */ | /* whitespace */)
/// ```
///
/// Recovery:
/// ```dbuf
/// [not (/* one comment */ message | /* one comment */ enum)]
/// ```
///
pub fn file_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let message = message_parser().map_child().map(|c| vec![c]);
    let enum_parser = enum_parser().map_child().map(|c| vec![c]);
    let comment = WhiteSpaceConfig::new()
        .with_limit_comment()
        .with_progress()
        .parser();
    let ws = WhiteSpaceConfig::new().with_progress().parser();

    let recovery_on = comment
        .clone()
        .or_not()
        .then(choice((just(Token::Message), just(Token::Enum))));
    let recovery_skip = any()
        .map_token()
        .and_is(recovery_on.not())
        .repeated()
        .collect::<Vec<_>>();
    let recovery = any()
        .map_token()
        .then(recovery_skip)
        .map_tree(TreeKind::ErrorTree)
        .map_child()
        .map(|c| vec![c]);

    let one_block = choice((message, enum_parser, comment, ws)).recover_with(via_parser(recovery));

    one_block
        .repeated()
        .collect::<Vec<_>>()
        .map_tree(TreeKind::File)
}

/// Parses one message.
///
/// Pattern:
/// ```dbuf
/// /*one comment*/
/// message /*comments*/ UCIdentifier /*comments*/ [<dependency definition> /* comments */]
///   <body>
/// ```
fn message_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let one_comment = WhiteSpaceConfig::new().with_limit_comment().parser();
    let ws = WhiteSpaceConfig::new().parser();
    let message_kw = just(Token::Message).map_token();
    let type_ident = type_identifier_parser();
    let dependency = dependency_parser().then(ws.clone());
    let dependency_r = dependency.repeated().collect::<Vec<_>>();
    let body = body_parser();

    one_comment
        .or_not()
        .then(message_kw)
        .then(ws.clone())
        .then(type_ident)
        .then(ws)
        .then(dependency_r)
        .then(body)
        .map_tree(TreeKind::Message)
}

/// Parses one dependency.
///
/// Pattern:
/// ```dbuf
/// (/* comments (all) */ <definition> /* comments */)
/// ```
fn dependency_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let ws = WhiteSpaceConfig::new().parser();

    let lparen = just(Token::LParen).map_token();
    let definition = definition_parser();
    let rparen = just(Token::RParen).map_token();

    lparen
        .then(ws.clone())
        .then(definition)
        .then(ws)
        .then(rparen)
        .map_with(|((((lp, mut comm1), mut tree), mut comm2), rp), extra| {
            let mut ans = vec![lp];
            ans.append(&mut comm1);
            ans.append(&mut tree.children);
            ans.append(&mut comm2);
            ans.push(rp);
            tree.children = ans;
            tree.location = extra.span();
            tree
        })
        .boxed()
}

/// Parses body.
///
/// Pattern:
/// ```dbuf
/// {
///   [ <field> | /* comment */ | /* whitespaces */ ]
/// }
/// ```
fn body_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let ws = WhiteSpaceConfig::new()
        .with_limit_comment()
        .with_progress()
        .parser();

    let lbrace = just(Token::LBrace).map_token();
    let rbrace = just(Token::RBrace).map_token();

    let field = field_parser().map_child().map(|c| vec![c]);
    let inside = choice((field, ws))
        .repeated()
        .collect::<Vec<_>>()
        .map(Vec::flatten);

    lbrace
        .then(inside)
        .then(rbrace)
        .map_tree(TreeKind::Body)
        .labelled("Body")
        .boxed()
}

/// Parses field.
///
/// Pattern:
/// ```dbuf
/// /* one comment */
/// <definition> /* comments */;
/// ```
fn field_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let one_comment = WhiteSpaceConfig::new().with_limit_comment().parser();
    let ws = WhiteSpaceConfig::new().parser();

    let definition = definition_parser();

    let semicolon = just(Token::Semicolon).map_token().map(Option::Some);
    let no_semicolon = semicolon.clone().not().rewind().map_with(|(), extra| {
        let loc = extra.span();
        let s: &mut ParsingState = extra.state();
        let def = s.definition_start.unwrap_or(loc);
        extra.emit(ParsingError::new(None, loc).missing_comma(def));
        None
    });
    let maybe_semicolon = semicolon.or(no_semicolon);

    one_comment
        .or_not()
        .then(definition.map_with(|d, extra| {
            let l = extra.span();
            extra.state().definition_start = Some(l);
            d
        }))
        .then(ws)
        .then(maybe_semicolon)
        .map_with(|(((comm1, mut tree), comm2), t), extra| {
            let mut nchildren = vec![];
            nchildren.extend(comm1.into_iter().flatten());
            nchildren.extend(tree.children);
            nchildren.extend(comm2);
            nchildren.extend(t);
            tree.children = nchildren;
            tree.location = extra.span();
            extra.state().definition_start = None;
            tree
        })
        .labelled("Field")
}

/// Parses definition.
///
/// Pattern:
/// ```dbuf
/// lcIdentifier /* comments */ UCIdentifier /* comments */
///   [/* comments */ (<parened expression>|<var chain>|<literal>|<constructed value>|<hole>)]
/// ```
fn definition_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let ws = WhiteSpaceConfig::new().with_no_new_line().parser();

    let var_ident = var_identifier_parser();
    let type_ident = type_identifier_parser();

    let expr = expression_parser();
    let paren_expr = parened_expression_parser(expr);
    let chain = var_chain_parser();
    let literal = literal_parser().map_tree(TreeKind::ExprLiteral);
    let cv = constructed_value_parser();
    let hole = typed_hole_parser();

    let arguments = choice((paren_expr, chain, literal, cv, hole));
    let commented_arguments = ws.clone().then(arguments).repeated().collect::<Vec<_>>();

    var_ident
        .then(ws.clone())
        .then(type_ident)
        .then(ws)
        .then(commented_arguments)
        .map_tree(TreeKind::Definition)
        .labelled("Definition")
}

/// Parses constructed value.
///
/// Pattern:
/// ```dbuf
/// UCIdentifier /* c */{[/* c */ lcIdentifier /* c */: /* c */ (<expression>|<constructed value>) /* c */,] | /* c */}
/// ```
fn constructed_value_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    recursive(constructed_value_parser_impl)
}

/// Implementation of constructed value parser.
fn constructed_value_parser_impl<'src, I>(
    cv_parser: impl Parser<'src, I, Tree, ExtraData> + Clone,
) -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let ws = WhiteSpaceConfig::new().parser();

    let type_indent = type_identifier_parser();
    let lbrace = just(Token::LBrace).map_token();
    let var_indent = var_identifier_parser();
    let colon = just(Token::Colon).map_token();
    let expr = expression_parser();
    let comma = just(Token::Comma).map_token();
    let rbrace = just(Token::RBrace).map_token();

    let field_value = choice((cv_parser, expr));

    let field_init = ws
        .clone()
        .then(var_indent)
        .then(ws.clone())
        .then(colon)
        .then(ws.clone())
        .then(field_value)
        .then(ws.clone())
        .then(comma.or_not())
        .map_tree(TreeKind::ConstructedValueField)
        .map_child();

    let field_r = field_init.repeated().at_least(1).collect::<Vec<_>>();

    type_indent
        .then(ws.clone())
        .then(lbrace)
        .then(choice((field_r, ws.clone())))
        .then(ws.clone())
        .then(rbrace)
        .map_tree(TreeKind::ConstructedValue)
}

/// Parses expressions.
///
/// Pattern is one of:
/// ```dbuf
/// /* comment */ <literal> /* comment */
/// /* comment */ <var chain> /* comment */
/// /* comment */ <typed hole> /* comment */
/// /* comment */ (/* comment */ <expression> /* comment */) /* comment */
/// <lhs_expression> (+|-|*|/|'|'|&) <rhs_expression>
/// /* comment */ (-|!) <rhs_expression>
/// ```
///
/// Note: Comments are tied to atom expressions so
/// ```dbuf
/// ( /* c1 */ (/* c2 */ 1 /* c3 */ + /* c4 */ 2 /* c5 */) /* c6 */ )
/// ```
///
/// parses to
/// ```dbuf
///           [/* c1 */ ( ) /* c6 */]
///                      |
///             [        +        ]
///            /                   \
/// [/* c2 */ 1 /* c3 */] [/* c4 */ 2 /* c5 */]
/// ```
fn expression_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    recursive(expression_parser_impl)
}

/// Implementation of expression parser.
fn expression_parser_impl<'src, I>(
    e_parser: impl Parser<'src, I, Tree, ExtraData> + Clone,
) -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let ws = WhiteSpaceConfig::new().with_no_new_line().parser();

    let literal_atom = literal_parser().map_tree(TreeKind::ExprLiteral);
    let identifier_atom = var_chain_parser();
    let parented_atom = parened_expression_parser(e_parser);
    let hole_atom = typed_hole_parser();

    let atom = choice((literal_atom, identifier_atom, parented_atom, hole_atom));

    let commented_atom =
        ws.clone()
            .then(atom)
            .then(ws.clone())
            .map_with(|((c1, mut a), mut c2), extra| {
                let mut nchildren = c1;
                nchildren.append(&mut a.children);
                nchildren.append(&mut c2);
                Tree {
                    kind: a.kind,
                    location: extra.span(),
                    children: nchildren,
                }
            });

    let binary_op = |c| just(c).map_token();
    let unary_op = |c| ws.clone().then(just(c).map_token());

    macro_rules! binary_fold {
        () => {
            |lhs, op, rhs, extra| Tree {
                kind: TreeKind::ExprBinary,
                location: extra.span(),
                children: vec![Child::Tree(lhs), op, Child::Tree(rhs)],
            }
        };
    }

    macro_rules! unary_fold {
        () => {
            |(comm, op), rhs, extra| {
                let mut children: Vec<Child> = comm;
                children.push(op);
                children.push(Child::Tree(rhs));
                Tree {
                    kind: TreeKind::ExprUnary,
                    location: extra.span(),
                    children,
                }
            }
        };
    }

    commented_atom
        .pratt((
            prefix(5, unary_op(Token::Minus), unary_fold!()),
            prefix(5, unary_op(Token::Bang), unary_fold!()),
            infix(left(4), binary_op(Token::Star), binary_fold!()),
            infix(left(4), binary_op(Token::Slash), binary_fold!()),
            infix(left(3), binary_op(Token::Plus), binary_fold!()),
            infix(left(3), binary_op(Token::Minus), binary_fold!()),
            infix(left(2), binary_op(Token::Amp), binary_fold!()),
            infix(left(1), binary_op(Token::Pipe), binary_fold!()),
        ))
        .labelled("Expression")
}

/// Parses parened expression.
///
/// Argument:
///     * `e_parser` is an expression parser.
///
/// Pattern:
/// ```dbuf
/// (<expression>)
/// ```
fn parened_expression_parser<'src, I>(
    e_parser: impl Parser<'src, I, Tree, ExtraData> + Clone,
) -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let l_paren = just(Token::LParen).map_token();
    let r_paren = just(Token::RParen).map_token();

    l_paren
        .then(e_parser)
        .then(r_paren)
        .map_tree(TreeKind::ExprParen)
        .labelled("Parened Expression")
}

/// Parses one enum.
///
/// Pattern:
/// ```dbuf
/// /*one comment*/
/// enum /* comments */ UCIdentifier /*comments*/ [<dependency definition> /* comments */]
///     <enumBody>
/// ```
fn enum_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let one_comment = WhiteSpaceConfig::new().with_limit_comment().parser();
    let ws = WhiteSpaceConfig::new().parser();

    let enum_kw = just(Token::Enum).map_token();
    let type_ident = type_identifier_parser();
    let dependency = dependency_parser().then(ws.clone());
    let dependency_r = dependency.repeated().collect::<Vec<_>>();
    let enum_body = enum_body_parser();

    one_comment
        .or_not()
        .then(enum_kw)
        .then(ws.clone())
        .then(type_ident)
        .then(ws)
        .then(dependency_r)
        .then(enum_body)
        .map_tree(TreeKind::Enum)
}

/// Parses enum body.
///
/// Pattern is:
/// ```dbuf
/// {
///     [<branch> | /* comment */ | /* whitespace */]
/// }
/// ```
/// or
/// ```dbuf
/// <constructor_enum>
/// ```
fn enum_body_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let ws = WhiteSpaceConfig::new()
        .with_limit_comment()
        .with_progress()
        .parser();

    let lbrace = just(Token::LBrace).map_token();
    let rbrace = just(Token::RBrace).map_token();

    let branch = branch_parser().map_child().map(|c| vec![c]);
    let inside = choice((branch, ws))
        .repeated()
        .collect::<Vec<_>>()
        .map(Vec::flatten);
    let branched = lbrace
        .then(inside)
        .then(rbrace)
        .map_tree(TreeKind::EnumBody);

    let ce = constructor_enum_parser()
        .map_child()
        .map_tree(TreeKind::EnumBody);

    choice((branched, ce))
}

/// Branch parser.
///
/// Pattern is
/// ```dbuf
/// <pattern> /* comment */ [, /* comment */ <pattern> /* comment */] => /* comment_r */ <constructor_enum>
/// ```
fn branch_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let ws = WhiteSpaceConfig::new().parser();

    let pattern = pattern_parser();
    let comma = just(Token::Comma).map_token();
    let arrow = just(Token::Arrow).map_token();
    let ce = constructor_enum_parser();

    let pattern_r = comma
        .then(ws.clone())
        .then(pattern.clone())
        .then(ws.clone())
        .repeated()
        .collect::<Vec<_>>();

    pattern
        .then(ws.clone())
        .then(pattern_r)
        .then(arrow)
        .then(ws)
        .then(ce)
        .map_tree(TreeKind::Branch)
}

/// Pattern parser.
///
/// Pattern is
/// ```dbuf
///     (* | <literal> | lcIdentifier |
///      UCIdentifier /* c */ { [/* c */ lcIdentifier /* c */ : /* c */ <pattern> /* c */, /* c */] | /* c */ } )
/// ```
fn pattern_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    recursive(pattern_parser_impl)
}

/// Implementation of pattern parser.
fn pattern_parser_impl<'src, I>(
    p_parser: impl Parser<'src, I, Tree, ExtraData> + Clone,
) -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let ws = WhiteSpaceConfig::new().parser();

    let star = just(Token::Star).map_token();
    let literal = literal_parser();
    let alias = var_identifier_parser();

    let type_indent = type_identifier_parser();
    let lbrace = just(Token::LBrace).map_token();
    let colon = just(Token::Colon).map_token();
    let comma = just(Token::Comma).map_token();
    let rbrace = just(Token::RBrace).map_token();

    let field_init = ws
        .clone()
        .then(alias.clone())
        .then(ws.clone())
        .then(colon)
        .then(ws.clone())
        .then(p_parser)
        .then(ws.clone())
        .then(comma.or_not())
        .then(ws.clone())
        .map_tree(TreeKind::ConstructedPatternField)
        .map_child();

    let field_r = field_init.repeated().at_least(1).collect::<Vec<_>>();

    let constructed_pattern = type_indent
        .then(ws.clone())
        .then(lbrace)
        .then(choice((field_r, ws)))
        .then(rbrace)
        .map_tree(TreeKind::ConstructedPattern)
        .map_child();

    choice((star, literal, alias, constructed_pattern)).map_tree(TreeKind::Pattern)
}

/// Parses constructor enum.
///
/// Pattern is:
/// ```dbuf
/// {
///    [<Constructor> | /* comment */ | /* whitespace */]
/// }
/// ```
fn constructor_enum_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let ws = WhiteSpaceConfig::new()
        .with_limit_comment()
        .with_progress()
        .parser();

    let lbrace = just(Token::LBrace).map_token();
    let rbrace = just(Token::RBrace).map_token();

    let constructor = constructor_parser().map_child().map(|c| vec![c]);
    let inside = choice((constructor, ws))
        .repeated()
        .collect::<Vec<_>>()
        .map(Vec::flatten);

    lbrace
        .then(inside)
        .then(rbrace)
        .map_tree(TreeKind::ConstructorEnum)
}

/// Parses constructor.
///
/// Pattern is:
/// ```dbuf
/// /* one comment */
/// UCIdentifier /* comment */
///     <body>
/// ```
/// or
/// ```dbuf
/// /* one comment */
/// UCIdentifier
/// ```
fn constructor_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let one_comment = WhiteSpaceConfig::new().with_limit_comment().parser();
    let ws = WhiteSpaceConfig::new().parser();
    let type_indent = type_identifier_parser();
    let body = body_parser();

    let constructor_with_body = one_comment
        .clone()
        .then(type_indent.clone())
        .then(ws)
        .then(body)
        .map_tree(TreeKind::Constructor);

    let constructor_no_body = one_comment
        .then(type_indent)
        .map_tree(TreeKind::Constructor);

    choice((constructor_with_body, constructor_no_body))
}

/// Configuration for whitespace parser.
#[allow(
    clippy::struct_excessive_bools,
    reason = "That not a state machine and all states are correct"
)]
struct WhiteSpaceConfig {
    /// consume regular spaces.
    consume_space: bool,
    /// consume new lines.
    consume_newline: bool,
    /// consume block comments.
    consume_block_comment: bool,
    /// consume line comments.
    consume_line_comment: bool,
    /// consume error tokens.
    consume_error: bool,
    /// require at least one success token.
    progress: bool,
    /// consume comment tokens no more that once.
    limit_comment: bool,
}

impl WhiteSpaceConfig {
    fn new() -> WhiteSpaceConfig {
        WhiteSpaceConfig {
            consume_space: true,
            consume_newline: true,
            consume_block_comment: true,
            consume_line_comment: true,
            consume_error: true,
            progress: false,
            limit_comment: false,
        }
    }

    fn with_progress(mut self) -> WhiteSpaceConfig {
        self.progress = true;
        self
    }

    fn with_limit_comment(mut self) -> WhiteSpaceConfig {
        self.limit_comment = true;
        self
    }

    fn with_no_new_line(mut self) -> WhiteSpaceConfig {
        self.consume_newline = false;
        self
    }

    fn parser<'src, I>(self) -> impl Parser<'src, I, Vec<Child>, ExtraData> + Clone
    where
        I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
    {
        let space = just(Token::Space).map_token().labelled("Space");

        let new_line = just(Token::NewLine).map_token().labelled("New Line");

        let line_comment = select! {
            Token::LineComment(comment) => Token::LineComment(comment)
        }
        .map_token()
        .labelled("Line Comment");

        let block_comment = select! {
            Token::BlockComment(comment) => Token::BlockComment(comment)
        }
        .map_token()
        .labelled("Block Comment");

        let error = just(Token::Err).map_token().labelled("Error");

        let mut comment_parser = any().filter(|_| false).map(|_| unreachable!()).boxed();
        if self.consume_block_comment {
            comment_parser = block_comment.or(comment_parser).boxed();
        }
        if self.consume_line_comment {
            comment_parser = line_comment.or(comment_parser).boxed();
        }

        let mut ws = any().filter(|_| false).map(|_| unreachable!()).boxed();
        if self.consume_space {
            ws = space.or(ws).boxed();
        }
        if self.consume_newline {
            ws = new_line.or(ws).boxed();
        }
        if self.consume_error {
            ws = error.or(ws).boxed();
        }

        let no_lim = choice((comment_parser.clone(), ws.clone()))
            .repeated()
            .collect::<Vec<_>>();
        let limited = ws
            .clone()
            .repeated()
            .collect::<Vec<_>>()
            .then(comment_parser.or_not())
            .then(ws.repeated().collect::<Vec<_>>())
            .map(|((mut ws1, comm), ws2)| {
                ws1.extend(comm);
                ws1.extend(ws2);
                ws1
            });

        match (self.progress, self.limit_comment) {
            (true, true) => limited.filter(|v| !v.is_empty()).boxed(),
            (true, false) => no_lim.filter(|v| !v.is_empty()).boxed(),
            (false, true) => limited.boxed(),
            (false, false) => no_lim.boxed(),
        }
    }
}

/// Parses type identifier (`UCIdentifier`).
fn type_identifier_parser<'src, I>() -> impl Parser<'src, I, Child, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    select! {
        Token::UCIdentifier(name) => Token::UCIdentifier(name)
    }
    .map_token()
    .labelled("Type Identifier")
}

/// Parses var chain.
///
/// Pattern:
/// ```dbuf
/// lcIdentifier [.lcIdentifier] (?= not .)
/// ```
///
/// Recovery:
/// ```dbuf
/// lcIdentifier [.lcIdentifier] .
/// ```
fn var_chain_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    let var_ident = var_identifier_parser();
    let dot = just(Token::Dot).map_token();
    let dot_call = dot
        .clone()
        .then(var_ident.clone())
        .repeated()
        .collect::<Vec<_>>();

    // recover on extra dot
    let finish = dot
        .clone()
        .map_with(|ch, extra| {
            let l = extra.span();
            let st = extra.state();
            let from = st.call_chain_start.unwrap_or(l);
            extra.emit(
                ParsingError::new(Token::Dot.into(), l)
                    .bad_call_chain((from..l).try_into().unwrap()),
            );
            ch
        })
        .or_not();

    var_ident
        .map_with(|ch, extra| {
            let loc = extra.span();
            let state = extra.state();
            state.call_chain_start = Some(loc);
            ch
        })
        .then(dot_call)
        .then(finish)
        .map_with(|ch, extra| {
            extra.state().call_chain_start = None;
            ch
        })
        .map_tree(TreeKind::ExprIdentifier)
}

/// Parses var identifier (`LCIdentifier`).
fn var_identifier_parser<'src, I>() -> impl Parser<'src, I, Child, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    select! {
        Token::LCIdentifier(name) => Token::LCIdentifier(name)
    }
    .map_token()
    .labelled("Variable Identifier")
}

/// Parses literals.
fn literal_parser<'src, I>() -> impl Parser<'src, I, Child, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    select! {
        Token::BoolLiteral(b) => Token::BoolLiteral(b),
        Token::IntLiteral(i) => Token::IntLiteral(i),
        Token::UintLiteral(ui) => Token::UintLiteral(ui),
        Token::FloatLiteral(f) => Token::FloatLiteral(f),
        Token::StringLiteral(s) => Token::StringLiteral(s),
    }
    .map_token()
    .labelled("Literal")
}

fn typed_hole_parser<'src, I>() -> impl Parser<'src, I, Tree, ExtraData> + Clone
where
    I: ValueInput<'src, Span = Location<Offset>, Token = Token>,
{
    just(Token::Underscore)
        .map_token()
        .map_tree(TreeKind::ExprHole)
        .map_with(|t, extra| {
            let err: ParsingError = extra.into();
            extra.emit(err.typed_hole());
            t
        })
}
