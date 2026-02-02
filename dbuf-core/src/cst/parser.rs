use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::pratt::*;
use chumsky::prelude::*;

use super::Location;
use super::{Child, Token, Tree, TreeKind};

use super::parser_error::ParsingError;
use super::parser_utils::{ChildFlatten, MapChild, MapToken, MapTree};

/// Parses one dbuf file.
///
pub fn file_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let message = message_parser().map_child().map(Option::Some);
    let comment = comment_parser().map(Option::Some);
    let ws = white_space_parser(true).map(|()| Option::None);

    choice((message, comment, ws))
        .repeated()
        .collect::<Vec<_>>()
        .map_with(|v, extra| Tree {
            kind: TreeKind::File,
            location: extra.span(),
            children: v.flatten(),
        })
}

/// Parses one message.
///
/// Pattern:
/// ```dbuf
/// /*one comment*/
/// message /*comments*/ UCIdentifier /*comments*/ [<dependency definition> /* comments */]
///   <body>
/// ```
fn message_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let one_comment = comment_one_parser();
    let comment_r = comment_r_parser();
    let message_kw = just(Token::Message).map_token();
    let type_ident = type_identifier_parser();
    let dependency = dependency_parser().then(comment_r.clone());
    let dependency_r = dependency.repeated().collect::<Vec<_>>();
    let body_parser = body_parser();

    one_comment
        .or_not()
        .then(message_kw)
        .then(comment_r.clone())
        .then(type_ident)
        .then(comment_r)
        .then(dependency_r)
        .then(body_parser)
        .map_tree(TreeKind::Message)
}

/// Parses one dependency.
///
/// Pattern:
/// ```dbuf
/// (/* comments (all) */ <definition> /* comments */)
/// ```
fn dependency_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let lparen = just(Token::LParen).map_token();
    let comment_r = comment_r_parser();
    let definition = definition_parser();
    let rparen = just(Token::RParen).map_token();

    lparen
        .then(comment_r.clone())
        .then(definition)
        .then(comment_r)
        .then(rparen)
        .map(|((((lp, mut comm1), mut tree), mut comm2), rp)| {
            let mut ans = vec![lp];
            ans.append(&mut comm1);
            ans.append(&mut tree.children);
            ans.append(&mut comm2);
            ans.push(rp);
            tree.children = ans;
            tree
        })
}

/// Parses body.
///
/// Pattern:
/// ```dbuf
/// {
///   [ <field> | /* comment */ | /* whitespaces */ ]
/// }
/// ```
fn body_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let lbrace = just(Token::LBrace).map_token();
    let field = field_parser().map_child().map(Option::Some);
    let comment = comment_parser().map(Option::Some);
    let rbrace = just(Token::RBrace).map_token();

    let ws = white_space_parser(true).map(|()| Option::None);

    let inside = choice((field, comment, ws)).repeated().collect::<Vec<_>>();

    lbrace.then(inside).then(rbrace).map_tree(TreeKind::Body)
}

/// Parses field.
///
/// Pattern:
/// ```dbuf
/// <definition> /* comments */;
/// ```
fn field_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let definition = definition_parser();
    let comment_r = comment_r_parser();
    let semicolon = just(Token::Semicolon).map_token();

    definition
        .then(comment_r)
        .then(semicolon)
        .map(|((mut tree, mut comm), t)| {
            tree.children.append(&mut comm);
            tree.children.push(t);
            tree
        })
}

/// Parses definition.
///
/// Pattern:
/// ```dbuf
/// /*one comment*/
/// lcIdentifier /* comments */ UCIdentifier /* comments */
///   [/* comments */ (<parened expression>|<var chain>|<literal>|<constructed value>)]
/// ```
fn definition_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let one_comment = comment_one_parser();
    let comment_r = comment_r_parser();
    let var_ident = var_identifier_parser();
    let type_ident = type_identifier_parser();

    let expr = expression_parser();
    let paren_expr = parened_expression_parser(expr).map_child();
    let chain = var_chain_parser().map_child();
    let literal = literal_parser();
    let cv = constructed_value_parser().map_child();

    let arguments = choice((paren_expr, chain, literal, cv));
    let commented_arguments = comment_r
        .clone()
        .then(arguments)
        .repeated()
        .collect::<Vec<_>>();

    one_comment
        .or_not()
        .then(var_ident)
        .then(comment_r.clone())
        .then(type_ident)
        .then(comment_r)
        .then(commented_arguments)
        .map_tree(TreeKind::Definition)
}

/// Parses constructed value.
///
/// Pattern:
/// ```dbuf
/// UCIdentifier /* c */{[/* c */ lcIdentifier /* c */: /* c */ (<expresssion>|<constructed value>) /* c */,]}
/// ```
fn constructed_value_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    recursive(constructed_value_parser_impl)
}

/// Implementation of constructed value parser.
fn constructed_value_parser_impl<'src, I>(
    cv_parser: impl Parser<'src, I, Tree, Err<ParsingError>> + Clone,
) -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let type_indent = type_identifier_parser();
    let comment_r = comment_r_parser();
    let lbrace = just(Token::LBrace).map_token();
    let var_indent = var_identifier_parser();
    let colon = just(Token::Colon).map_token();
    let expr = expression_parser();
    let comma = just(Token::Comma).map_token();
    let rbrace = just(Token::RBrace).map_token();

    let field_value = choice((cv_parser, expr));

    let field_init = comment_r
        .clone()
        .then(var_indent)
        .then(comment_r.clone())
        .then(colon)
        .then(comment_r.clone())
        .then(field_value)
        .then(comment_r.clone())
        .then(comma.or_not())
        .map_tree(TreeKind::ConstructedValueField)
        .map_child();

    let field_r = field_init.repeated().at_least(1).collect::<Vec<_>>();

    type_indent
        .then(comment_r.clone())
        .then(lbrace)
        .then(choice((field_r, comment_r.clone())))
        .then(comment_r.clone())
        .then(rbrace)
        .map_tree(TreeKind::ConstructedValue)
}

/// Parses expressions.
///
/// Pattern is one of:
/// ```dbuf
/// /* comment */ <literal> /* comment */
/// /* comment */ <var chain> /* comment */
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
fn expression_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    recursive(expression_parser_impl)
}

/// Implementation of expression parser.
fn expression_parser_impl<'src, I>(
    e_parser: impl Parser<'src, I, Tree, Err<ParsingError>> + Clone,
) -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let literal_atom = literal_parser().map_tree(TreeKind::ExprLiteral);
    let identifier_atom = var_chain_parser();
    let parented_atom = parened_expression_parser(e_parser);

    let atom = choice((literal_atom, identifier_atom, parented_atom));
    let comment_r = comment_r_parser();

    let commented_atom =
        comment_r
            .clone()
            .then(atom)
            .then(comment_r.clone())
            .map(|((c1, mut a), mut c2)| {
                let mut nchildren = c1;
                nchildren.append(&mut a.children);
                nchildren.append(&mut c2);
                Tree {
                    kind: a.kind,
                    location: a.location,
                    children: nchildren,
                }
            });

    let binary_op = |c| just(c).map_token();
    let unary_op = |c| comment_r.clone().then(just(c).map_token());

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
    e_parser: impl Parser<'src, I, Tree, Err<ParsingError>> + Clone,
) -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let l_paren = just(Token::LParen).map_token();
    let r_paren = just(Token::RParen).map_token();

    l_paren
        .then(e_parser)
        .then(r_paren)
        .map_tree(TreeKind::ExprParen)
        .labelled("Parened Expression")
}

/// Parses and ignores any number of whitespaces.
///
/// Param:
///     `progress`. If true, require at least one white space.
fn white_space_parser<'src, I>(
    progress: bool,
) -> impl Parser<'src, I, (), Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let white_space = choice((just(Token::Newline), just(Token::Space)));

    let at_least = usize::from(progress);
    white_space.repeated().at_least(at_least)
}

/// Parses one comment with spaces.
fn comment_parser<'src, I>() -> impl Parser<'src, I, Child, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
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

    let ws = white_space_parser(false);

    ws.clone()
        .then(choice((line_comment, block_comment)))
        .then(ws)
        .map(|(((), c), ())| c)
}

/// Parses any number of comments/spaces.
fn comment_r_parser<'src, I>() -> impl Parser<'src, I, Vec<Child>, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let ws = white_space_parser(false).map(|()| vec![]);

    let comment = comment_parser();
    let comment_r = comment.repeated().at_least(1).collect::<Vec<_>>();

    choice((comment_r, ws))
}

/// Parsees one or zero comments with spaces.
fn comment_one_parser<'src, I>() -> impl Parser<'src, I, Option<Child>, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let ws = white_space_parser(false).map(|()| Option::None);
    let comment = comment_parser().map(Option::Some);

    choice((comment, ws))
}

/// Parses type identifier (`UCIdentifier`).
fn type_identifier_parser<'src, I>() -> impl Parser<'src, I, Child, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
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
fn var_chain_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let var_ident = var_identifier_parser();
    let dot = just(Token::Dot).map_token();
    let dot_call = dot
        .clone()
        .then(var_ident.clone())
        .repeated()
        .collect::<Vec<_>>();

    let recovery = var_ident
        .clone()
        .then(dot_call.clone())
        .then(dot.clone())
        .map_tree(TreeKind::ExprIdentifier);

    var_ident
        .then(dot_call)
        .then(dot.not().rewind())
        .map_tree(TreeKind::ExprIdentifier)
        .map_err(ParsingError::bad_call_chain)
        .recover_with(via_parser(recovery))
}

/// Parses var identifier (`LCIdentifier`).
fn var_identifier_parser<'src, I>() -> impl Parser<'src, I, Child, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    select! {
        Token::LCIdentifier(name) => Token::LCIdentifier(name)
    }
    .map_token()
    .labelled("Variable Identifier")
}

/// Parses literals.
fn literal_parser<'src, I>() -> impl Parser<'src, I, Child, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
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
