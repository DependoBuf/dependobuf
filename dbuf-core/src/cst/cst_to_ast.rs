//! Module exports:
//!   * `convert` function, which converts CST to AST
use crate::ast::{
    operators::*,
    parsed::{self, definition::*, location, *},
};

use super::{Child, Token, Tree, TreeKind};

type LocationCST = super::Location;

type OffsetAST = location::Offset;
type LocationAST = location::Location<OffsetAST>;
type NameAST = parsed::located_name::LocatedName<String, OffsetAST>;

pub type ParsedModule = Module<LocationAST, NameAST>;

/// Converts `CST` file to `ParsedModule`.
pub fn convert(file: &Tree) -> ParsedModule {
    match file.kind {
        TreeKind::File => convert_file(file),
        _ => vec![],
    }
}

#[derive(Clone, Copy)]
enum NameKind {
    UC,
    LC,
}
use NameKind::*;

fn to_name(child: &Child, kind: NameKind) -> Option<NameAST> {
    match kind {
        UC => match child {
            Child::Token(Token::UCIdentifier(name), loc) => Some(NameAST {
                content: name.to_owned(),
                start: loc.start(),
            }),
            _ => None,
        },
        LC => match child {
            Child::Token(Token::LCIdentifier(name), loc) => Some(NameAST {
                content: name.to_owned(),
                start: loc.start(),
            }),
            _ => None,
        },
    }
}

impl From<&Tree> for LocationAST {
    fn from(value: &Tree) -> Self {
        (&value.location).into()
    }
}

impl From<&LocationCST> for LocationAST {
    fn from(value: &LocationCST) -> Self {
        LocationAST {
            start: value.start(),
            length: (value.end() - value.start()).expect("correct location cst"),
        }
    }
}

fn convert_file(file: &Tree) -> ParsedModule {
    assert!(file.kind == TreeKind::File);

    let mut ans = vec![];

    for child in &file.children {
        let Child::Tree(t) = child else {
            continue;
        };
        match t.kind {
            TreeKind::Message => ans.push(convert_message(t)),
            TreeKind::Enum => ans.push(convert_enum(t)),
            _ => (),
        }
    }

    ans
}

fn convert_message(
    message: &Tree,
) -> Definition<LocationAST, NameAST, TypeDeclaration<LocationAST, NameAST>> {
    assert!(message.kind == TreeKind::Message);

    let mut dependencies = vec![];
    let mut body = None;
    let mut name = None;
    for child in &message.children {
        if let Some(n) = to_name(child, UC) {
            name = Some(n);
            continue;
        }
        let Child::Tree(t) = child else {
            continue;
        };
        match t.kind {
            TreeKind::Body => body = Some(convert_body(t)),
            TreeKind::Definition => dependencies.push(convert_definition(t)),
            _ => (),
        }
    }

    Definition {
        loc: message.into(),
        name: name.expect("UCIdentifier child in Message tree"),
        data: TypeDeclaration {
            dependencies,
            body: TypeDefinition::Message(body.expect("Body identifier child in Message tree")),
        },
    }
}

fn convert_definition(
    definition: &Tree,
) -> Definition<LocationAST, NameAST, TypeExpression<LocationAST, NameAST>> {
    assert!(definition.kind == TreeKind::Definition);

    let child_iter = definition.children.iter();

    let mut child_iter = child_iter.skip_while(|c| to_name(c, LC).is_none());
    let name = child_iter
        .next()
        .map(|c| to_name(c, LC).expect("iterated to name"))
        .expect("LCIdentifier child in Definition tree");

    let mut child_iter = child_iter.skip_while(|c| to_name(c, UC).is_none());
    let type_name = child_iter
        .next()
        .map(|c| to_name(c, UC).expect("iterated to name"))
        .expect("UCIdentifier child in Definition tree");

    let mut args = vec![];
    for child in child_iter {
        let Child::Tree(t) = child else {
            continue;
        };

        if !is_expression(t) {
            continue;
        }
        args.push(convert_expression(t));
    }

    let te_start = type_name.start;
    let mut te_end = type_name.end();
    if let Some(l) = args.last() {
        te_end = l.loc.end();
    }

    Definition {
        loc: definition.into(),
        name,
        data: TypeExpression {
            loc: LocationAST {
                start: te_start,
                length: (te_end - te_start).expect("correct range"),
            },
            node: ExpressionNode::FunCall {
                fun: type_name,
                args: args.into_boxed_slice().into(),
            },
        },
    }
}

fn is_expression(expression: &Tree) -> bool {
    matches!(
        expression.kind,
        TreeKind::ConstructedValue
            | TreeKind::ExprParen
            | TreeKind::ExprLiteral
            | TreeKind::ExprIdentifier
            | TreeKind::ExprBinary
            | TreeKind::ExprUnary
            | TreeKind::ExprHole
    )
}

fn convert_expression(expression: &Tree) -> Expression<LocationAST, NameAST> {
    assert!(is_expression(expression));

    let node = match expression.kind {
        TreeKind::ConstructedValue => {
            return convert_constructed_value(expression);
        }
        TreeKind::ExprParen => {
            for child in &expression.children {
                if let Child::Tree(t) = child
                    && is_expression(t)
                {
                    let mut ans = convert_expression(t);
                    ans.loc = expression.into();
                    return ans;
                }
            }
            panic!("bad parened expression");
        }
        TreeKind::ExprLiteral => {
            let literal = convert_literal(expression);
            ExpressionNode::OpCall(OpCall::Literal(
                literal.expect("literal in ExprLiteral tree"),
            ))
        }
        TreeKind::ExprIdentifier => {
            return convert_expression_identifier(expression);
        }
        TreeKind::ExprBinary => {
            let mut lhs = None;
            let mut op = None;
            let mut rhs = None;
            for child in &expression.children {
                if lhs.is_none() {
                    let Child::Tree(t) = child else {
                        continue;
                    };
                    if is_expression(t) {
                        lhs = convert_expression(t).into();
                    }
                    continue;
                }
                if op.is_none() {
                    let cur_op = match child {
                        Child::Token(Token::Plus, _) => BinaryOp::Plus,
                        Child::Token(Token::Minus, _) => BinaryOp::Minus,
                        Child::Token(Token::Star, _) => BinaryOp::Star,
                        Child::Token(Token::Slash, _) => BinaryOp::Slash,
                        Child::Token(Token::Amp, _) => BinaryOp::BinaryAnd,
                        Child::Token(Token::Pipe, _) => BinaryOp::BinaryOr,
                        _ => continue,
                    };
                    op = cur_op.into();
                    continue;
                }
                let Child::Tree(t) = child else {
                    continue;
                };
                if is_expression(t) {
                    rhs = convert_expression(t).into();
                    break;
                }
            }
            ExpressionNode::OpCall(OpCall::Binary(
                op.expect("op in ExprBinary tree"),
                lhs.expect("lhs expression in ExprBinary tree").into(),
                rhs.expect("rhs expression in ExprBinary tree").into(),
            ))
        }
        TreeKind::ExprUnary => {
            let mut op = None;
            let mut rhs = None;
            for child in &expression.children {
                if op.is_none() {
                    let cur_op = match child {
                        Child::Token(Token::Minus, _) => UnaryOp::Minus,
                        Child::Token(Token::Bang, _) => UnaryOp::Bang,
                        _ => continue,
                    };
                    op = cur_op.into();
                    continue;
                }
                let Child::Tree(t) = child else {
                    continue;
                };
                if is_expression(t) {
                    rhs = convert_expression(t).into();
                    break;
                }
            }
            ExpressionNode::OpCall(OpCall::Unary(
                op.expect("op in ExprUnary tree"),
                rhs.expect("rhs expression in ExprUnary tree").into(),
            ))
        }
        TreeKind::ExprHole => ExpressionNode::TypedHole,
        _ => panic!("bad expression tree kind"),
    };

    Expression {
        loc: expression.into(),
        node,
    }
}

fn convert_expression_identifier(ei: &Tree) -> Expression<LocationAST, NameAST> {
    assert!(ei.kind == TreeKind::ExprIdentifier);

    let mut ident = vec![];
    for child in &ei.children {
        if let Child::Token(Token::Dot, _) = child {
            ident.push(None);
            continue;
        }
        if let Some(n) = to_name(child, LC) {
            ident.push(n.into());
        }
    }

    let mut ident = ident.into_iter();

    let first = ident.next().expect("LCIdentifier in ExprIdentifier");
    let first_ident = first.expect("ExprIdentifier starts with not dot");

    let mut ans = Expression {
        loc: LocationAST {
            start: first_ident.start,
            length: (first_ident.end() - first_ident.start).expect("correct location"),
        },
        node: ExpressionNode::Variable { name: first_ident },
    };

    let start = ei.location.start();

    for i in ident {
        let Some(name) = i else {
            continue;
        };
        let end = name.end();
        let length = (end - start).expect("correct location");
        let loc = LocationAST { start, length };

        ans = Expression {
            loc,
            node: ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Access(name.clone()), ans.into())),
        };
    }

    ans
}

fn convert_literal(e: &Tree) -> Option<Literal> {
    assert!(e.kind == TreeKind::ExprLiteral || e.kind == TreeKind::Pattern);

    let mut literal = None;
    for child in &e.children {
        let Child::Token(t, _) = child else {
            continue;
        };
        literal = match t {
            Token::BoolLiteral(b) => Literal::Bool(*b),
            Token::IntLiteral(i) => Literal::Int(*i),
            Token::UintLiteral(ui) => Literal::UInt(*ui),
            Token::FloatLiteral(f) => Literal::Double(*f),
            Token::StringLiteral(s) => Literal::Str(s.clone()),
            _ => continue,
        }
        .into();
        break;
    }
    literal
}

fn convert_constructed_value(cv: &Tree) -> Expression<LocationAST, NameAST> {
    assert!(cv.kind == TreeKind::ConstructedValue);

    let mut name = None;
    let mut fields = vec![];
    for child in &cv.children {
        if let Some(n) = to_name(child, UC) {
            name = n.into();
            continue;
        }

        let Child::Tree(t) = child else {
            continue;
        };

        if t.kind == TreeKind::ConstructedValueField {
            fields.push(convert_constructed_value_field(t));
        }
    }

    Expression {
        loc: cv.into(),
        node: ExpressionNode::ConstructorCall {
            name: name.expect("UCIdentifier in ConstructedValue"),
            fields,
        },
    }
}

fn convert_constructed_value_field(
    field: &Tree,
) -> Definition<LocationAST, NameAST, Expression<LocationAST, NameAST>> {
    assert!(field.kind == TreeKind::ConstructedValueField);

    let mut name = None;
    for child in &field.children {
        if let Some(n) = to_name(child, LC) {
            name = n.into();
            continue;
        }
        let Child::Tree(t) = child else {
            continue;
        };
        if is_expression(t) {
            let expr = convert_expression(t);
            return Definition {
                name: name.expect("LCIdentifier in ConstructedValueField tree"),
                loc: field.into(),
                data: expr,
            };
        }
    }

    panic!("expected expression in ConstructedValueField tree");
}

fn convert_body(body: &Tree) -> ConstructorBody<LocationAST, NameAST> {
    assert!(body.kind == TreeKind::Body);

    let mut fields = vec![];
    for child in &body.children {
        let Child::Tree(t) = child else {
            continue;
        };

        if t.kind == TreeKind::Definition {
            fields.push(convert_definition(t));
        }
    }

    fields
}

fn convert_enum(
    e: &Tree,
) -> Definition<LocationAST, NameAST, TypeDeclaration<LocationAST, NameAST>> {
    assert!(e.kind == TreeKind::Enum);

    let mut dependencies = vec![];
    let mut body = None;
    let mut name = None;
    for child in &e.children {
        if let Some(n) = to_name(child, UC) {
            name = Some(n);
            continue;
        }
        let Child::Tree(t) = child else {
            continue;
        };
        match t.kind {
            TreeKind::Definition => dependencies.push(convert_definition(t)),
            TreeKind::EnumBody => body = Some(convert_enum_body(t)),
            _ => (),
        }
    }

    Definition {
        loc: e.into(),
        name: name.expect("UCIdentifier in Enum tree"),
        data: TypeDeclaration {
            dependencies,
            body: TypeDefinition::Enum(body.expect("EnumBody in Enum tree")),
        },
    }
}

fn convert_enum_body(body: &Tree) -> Vec<EnumBranch<LocationAST, NameAST>> {
    assert!(body.kind == TreeKind::EnumBody);

    let mut branches = vec![];

    for child in &body.children {
        let Child::Tree(t) = child else {
            continue;
        };

        match t.kind {
            TreeKind::Branch => branches.push(convert_branch(t)),
            TreeKind::ConstructorEnum => {
                let ctrs = convert_constructor_enum(t);
                branches.push(EnumBranch {
                    patterns: vec![],
                    constructors: ctrs,
                });
            }
            _ => (),
        }
    }

    branches
}

fn convert_branch(branch: &Tree) -> EnumBranch<LocationAST, NameAST> {
    assert!(branch.kind == TreeKind::Branch);

    let mut patterns = vec![];
    let mut constructors = vec![];
    for child in &branch.children {
        let Child::Tree(t) = child else {
            continue;
        };
        match t.kind {
            TreeKind::Pattern => {
                patterns.push(convert_pattern(t));
            }
            TreeKind::ConstructorEnum => {
                constructors = convert_constructor_enum(t);
            }
            _ => (),
        }
    }

    EnumBranch {
        patterns,
        constructors,
    }
}

fn convert_pattern(pattern: &Tree) -> Pattern<LocationAST, NameAST> {
    assert!(pattern.kind == TreeKind::Pattern);

    if let Some(literal) = convert_literal(pattern) {
        return Pattern {
            loc: pattern.into(),
            node: PatternNode::Literal(literal),
        };
    }

    for child in &pattern.children {
        if let Some(n) = to_name(child, LC) {
            return Pattern {
                loc: pattern.into(),
                node: PatternNode::Variable { name: n },
            };
        }
        if let Child::Token(Token::Star, _) = child {
            return Pattern {
                loc: pattern.into(),
                node: PatternNode::Underscore,
            };
        }
        let Child::Tree(t) = child else {
            continue;
        };

        if t.kind == TreeKind::ConstructedPattern {
            return convert_constructed_patter(t);
        }
    }

    panic!("Pattern tree with no pattern");
}

fn convert_constructed_patter(cp: &Tree) -> Pattern<LocationAST, NameAST> {
    assert!(cp.kind == TreeKind::ConstructedPattern);

    let mut name = None;
    let mut fields = vec![];
    for child in &cp.children {
        if let Some(n) = to_name(child, UC) {
            name = n.into();
        }

        let Child::Tree(t) = child else {
            continue;
        };

        if t.kind == TreeKind::ConstructedPatternField {
            fields.push(convert_constructed_patter_field(t));
        }
    }

    Pattern {
        loc: cp.into(),
        node: PatternNode::ConstructorCall {
            name: name.expect("UCIdentifier in ConstructedPattern tree"),
            fields,
        },
    }
}

fn convert_constructed_patter_field(
    field: &Tree,
) -> Definition<LocationAST, NameAST, Pattern<LocationAST, NameAST>> {
    assert!(field.kind == TreeKind::ConstructedPatternField);

    let mut name = None;

    for child in &field.children {
        if let Some(n) = to_name(child, LC) {
            name = n.into();
            continue;
        }

        let Child::Tree(t) = child else {
            continue;
        };

        if t.kind == TreeKind::Pattern {
            let p = convert_pattern(t);
            return Definition {
                loc: field.into(),
                name: name.expect("LCIdentifier in ConstructedPatternField tree"),
                data: p,
            };
        }
    }

    panic!("expected pattern in ConstructedPatternField tree");
}

fn convert_constructor_enum(
    ce: &Tree,
) -> Definitions<LocationAST, NameAST, ConstructorBody<LocationAST, NameAST>> {
    assert!(ce.kind == TreeKind::ConstructorEnum);

    let mut ctrs = vec![];
    for child in &ce.children {
        let Child::Tree(t) = child else {
            continue;
        };

        if t.kind == TreeKind::Constructor {
            ctrs.push(convert_constructor(t));
        }
    }

    ctrs
}

fn convert_constructor(
    constructor: &Tree,
) -> Definition<LocationAST, NameAST, ConstructorBody<LocationAST, NameAST>> {
    assert!(constructor.kind == TreeKind::Constructor);

    let mut name = None;
    let mut body = None;
    for child in &constructor.children {
        if let Some(n) = to_name(child, UC) {
            name = n.into();
            continue;
        }

        let Child::Tree(t) = child else {
            continue;
        };

        if t.kind == TreeKind::Body {
            body = convert_body(t).into();
        }
    }

    if body.is_none() {
        body = vec![].into();
    }

    Definition {
        loc: constructor.into(),
        name: name.expect("UCIdentifier in Constructor tree"),
        data: body.expect("Just set"),
    }
}
