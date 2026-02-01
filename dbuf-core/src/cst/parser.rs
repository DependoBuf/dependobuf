use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::pratt::*;
use chumsky::prelude::*;

use crate::cst::parser_utils::MapChild;

use super::Location;
use super::{Child, Token, Tree, TreeKind};

use super::parser_error::ParsingError;
use super::parser_utils::{MapToken, MapTree};

/// Parses one dbuf file.
///
pub fn file_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let message = message_parser().map_child();
    let comment = comment_parser();

    choice((message, comment))
        .repeated()
        .collect::<Vec<_>>()
        .map_with(|v, extra| Tree {
            kind: TreeKind::File,
            location: extra.span(),
            children: v,
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
    let comment = comment_parser();
    let comment_r = comment.clone().repeated().collect::<Vec<_>>();
    let message_kw = just(Token::Message).map_token();
    let type_ident = type_identifier_parser();
    let dependency = dependency_parser().then(comment_r.clone());
    let dependency_r = dependency.repeated().collect::<Vec<_>>();
    let body_parser = body_parser();

    comment
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
    let lparent = just(Token::LParen).map_token();
    let comment_r = comment_parser().repeated().collect::<Vec<_>>();
    let definition = definition_parser();
    let rparent = just(Token::RParen).map_token();

    lparent
        .then(comment_r.clone())
        .then(definition)
        .then(comment_r)
        .then(rparent)
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
///   [ <field> | /* comment */ ]
/// }
/// ```
fn body_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let lbrace = just(Token::LBrace).map_token();
    let field = field_parser().map_child();
    let comment = comment_parser();
    let rbrace = just(Token::RBrace).map_token();

    let inside = choice((field, comment)).repeated().collect::<Vec<_>>();

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
    let comment_r = comment_parser().repeated().collect::<Vec<_>>();
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
/// lcIdentifier /* comments */ UCIdentifier /* comments */ [/* comments */ <parened expression> | /* comments */ <var chain>]
/// ```
fn definition_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let comment = comment_parser();
    let comment_r = comment.clone().repeated().collect::<Vec<_>>();
    let var_ident = var_identifier_parser();
    let type_ident = type_identifier_parser();

    let expr = exrpession_parser();
    let paren_expr = parened_expression_parser(expr);
    let chain = var_chain_parser();

    let arguments = choice((paren_expr, chain));
    let commented_arguments = comment_r
        .clone()
        .then(arguments)
        .repeated()
        .collect::<Vec<_>>();

    comment
        .or_not()
        .then(var_ident)
        .then(comment_r.clone())
        .then(type_ident)
        .then(comment_r)
        .then(commented_arguments)
        .map_tree(TreeKind::Definition)
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
fn exrpession_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    recursive(exrpession_parser_impl)
}

/// Implementation of expression parser.
fn exrpession_parser_impl<'src, I>(
    e_parser: impl Parser<'src, I, Tree, Err<ParsingError>> + Clone,
) -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let literal_atom = literal_parser().map_tree(TreeKind::ExprLiteral);
    let identifier_atom = var_chain_parser();
    let parented_atom = parened_expression_parser(e_parser);

    let atom = choice((literal_atom, identifier_atom, parented_atom));
    let comment_r = comment_parser().repeated().collect::<Vec<_>>();

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

/// Parses one comment.
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

    choice((line_comment, block_comment))
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
/// lcIdentifier [.lcIdentifier]
/// ```
fn var_chain_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let var_ident = var_identifier_parser();
    let dot = just(Token::Dot).map_token();
    let dot_call = dot.then(var_ident.clone()).repeated().collect::<Vec<_>>();
    var_ident.then(dot_call).map_tree(TreeKind::ExprIdentifier)
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
