//! Convinient visitor for parsed ast.
//!
//! Module exports:
//! * trait Visitor, indicating, that object wants to visit parsed ast,
//! * enum Visit for parsed ast tokens.
//! * enum VisitResult for Visitor results for visit.
//! * fn visit_ast for invoking visitor.
//!
//! Module also contains other useful modules:
//! * mod scope visitor containing ScopeVisitor, wich helps with scope control
//!

pub mod scope_visitor;

use dbuf_core::ast::operators::*;
use dbuf_core::ast::parsed::*;

use super::ast_access::{Loc, Str};

use super::ast_access::ElaboratedAst;
use super::ast_access::ElaboratedHelper;
use super::ast_access::LocStringHelper;
use super::ast_access::ParsedAst;
use super::ast_access::Position;
use super::dbuf_language::is_correct_type_name;

/// Constructor characteristic. Do not confuse with constructor calls.
///
/// 'a is lifetime of parsed ast reference.
pub struct Constructor<'a> {
    /// Name of constructor
    pub name: &'a Str,
    /// Indicating if constructor constructs message
    pub of_message: bool,
    /// Location of whole constructor
    pub loc: &'a Loc,
}

/// Contains parsed ast tokens.
///
/// 'a is lifetime of parsed ast reference.
pub enum Visit<'a> {
    /// Keyword. Currently one of:
    /// * `message`,
    /// * `enum`.
    ///
    /// Can be skipped (to next type declaration).
    ///
    /// 0: keyword.
    ///
    /// 1: location.
    ///
    /// Location calculations is bad
    /// due to incomplete parsed ast.
    ///
    /// TODO:
    /// * better find location algorithm.
    Keyword(&'static str, Loc),
    /// Type declaration.
    ///
    /// Can be skipped (to next type declaration).
    ///
    /// 0: type name.
    ///
    /// 1: location of whole type.
    Type(&'a Str, &'a Loc),
    /// Type dependency.
    ///
    /// Can be skipped (to next dependency).
    ///
    /// 0: dependency name.
    ///
    /// 1: location of whole dependency.
    Dependency(&'a Str, &'a Loc),
    /// New branch in enum.
    ///
    /// Can be skipped (to next enum branch).
    Branch,
    /// Pattern alias.
    ///
    /// No skip allowed.
    ///
    /// 0: alias.
    PatternAlias(&'a Str),
    /// Constructor call in pattern.
    ///
    /// Can be skipped (to next pattern).
    ///
    /// 0: constructor name.
    ///
    /// 1: location of whole call.
    PatternCall(&'a Str, &'a Loc),
    /// Constructor field name.
    ///
    /// Can be skipped (to next call argument).
    ///
    /// 0: field name.
    ///
    /// Currently, always None
    /// due to incomplete parsed ast.
    ///
    /// TODO:
    /// * return not none.
    PatternCallArgument(Option<&'a Str>),
    /// Call end.
    ///
    /// No skip allowed.
    ///
    PatternCallStop,
    /// Literal in pattern.
    ///
    /// No skip allowed.
    ///
    /// 0: literal.
    ///
    /// 1: location.
    ///
    /// due to incomplete ast interface
    /// is not optimal. Changes are expected.
    ///
    /// TODO:
    /// * changes (?).
    PatternLiteral(&'a Literal, &'a Loc),
    /// Underscore in pattern.
    ///
    /// No skip allowed.
    ///
    /// 0: location.
    PatternUnderscore(&'a Loc),
    /// Constructor. Do not confuse with constructor call.
    ///
    /// Can be skipped (to next constructor).
    ///
    /// 0: constructor characteristic.
    Constructor(Constructor<'a>),
    /// Field.
    ///
    /// Can be skipped (to next field).
    ///
    /// 0: field name.
    ///
    /// 1: location of whole field.
    Filed(&'a Str, &'a Loc),
    /// Type expression.
    ///
    /// Can be skipped (to next dependency/field).
    ///
    /// 0: type name.
    ///
    /// 1: location of whole expression.
    TypeExpression(&'a Str, &'a Loc),
    /// Expression.
    ///
    /// Can be skipped (to next expression).
    ///
    /// 0: location of whole expression.
    Expression(&'a Loc),
    /// Start of access chain.
    ///
    /// Can be skipped (to next expression).
    AccessChainStart,
    /// Access in access chain.
    ///
    /// No skip allowed.
    ///
    /// 0: access.
    AccessChain(&'a Str),
    /// Dot in access chain.
    ///
    /// No skip allowed.
    ///
    /// 0: Location of dot.
    ///
    /// location calculation might be
    /// inaccurate due to incomplete ast.
    ///
    /// TODO:
    /// * better location calculation.
    AccessDot(Loc),
    /// Last access in access chain.
    ///
    /// No skip allowed.
    ///
    AccessChainLast(&'a Str),
    /// Constuctor call in expression.
    ///
    /// Can be skipped (to next expression).
    ///
    /// 0: constructor name.
    ConstructorExpr(&'a Str),
    /// Constructor field name.
    ///
    /// Can be skipped (to next argument).
    ///
    /// 0: constructor field name.
    ///
    /// Currently, always None
    /// due to incomplete parsed ast.
    ///
    /// TODO:
    /// * return not none.
    ConstructorExprArgument(Option<&'a Str>),
    /// Constructor call end.
    ///
    /// No skip allowed.
    ConstructorExprStop,
    /// Access.
    ///
    /// No skip allowed.
    ///
    /// 0: access
    VarAccess(&'a Str),
    /// Operator.
    ///
    /// No skip allowed.
    ///
    /// 0: operator string.
    ///
    /// 1: location of operator.
    ///
    /// Location might be inaccurate
    /// due to incomplete parsed ast.
    ///
    /// TODO:
    /// * better location calculation.
    Operator(&'static str, Loc),
    /// Literal in expression.
    ///
    /// No skip allowed.
    ///
    /// 0: literal.
    ///
    /// 1: its location.
    Literal(&'a Literal, &'a Loc),
}

impl<'a> From<Constructor<'a>> for Visit<'a> {
    fn from(value: Constructor<'a>) -> Self {
        Visit::Constructor(value)
    }
}

impl<'a> Visit<'a> {
    fn message_constructor(name: &'a Str, loc: &'a Loc) -> Visit<'a> {
        Constructor {
            name,
            of_message: true,
            loc,
        }
        .into()
    }
    fn enum_constructor(name: &'a Str, loc: &'a Loc) -> Visit<'a> {
        Constructor {
            name,
            of_message: false,
            loc,
        }
        .into()
    }
}

/// Visit result. One of:
/// * Continue - parse current subtree (if any),
/// * Skip - skip current subtree. Panics if no such.
/// * Stop - stop parsing.
pub enum VisitResult {
    Continue,
    Skip,
    Stop,
}

/// Visitor - the one, who can parse Visit tokens.
///
/// 'a is lifetime of parsed ast reference.
pub trait Visitor<'a> {
    fn visit(&mut self, visit: Visit<'a>) -> VisitResult;
}

/// Visit whole ast. Skips parts, if visitor tells so.
///
/// Currently takes elaborated ast as argument to generate
/// `message` and `enum` keywords. That's due incomplete
/// parsed tree.
///
/// Todo:
/// * remove elaborated ast argument.
pub fn visit_ast<'a, V: Visitor<'a>>(
    ast: &'a ParsedAst,
    visitor: &mut V,
    tempo_elaborated: &'a ElaboratedAst,
) {
    let mut stop_visit = false;

    for td in ast.iter() {
        let keyword = if tempo_elaborated.is_message(td.name.as_ref()) {
            get_keyword("message", td.name.get_location().start.line)
        } else {
            get_keyword("enum", td.name.get_location().start.line)
        };

        let res = visitor.visit(keyword);
        match res {
            VisitResult::Continue => {}
            VisitResult::Skip => continue,
            VisitResult::Stop => stop_visit = true,
        }

        if stop_visit {
            return;
        }

        let res = visitor.visit(Visit::Type(&td.name, &td.loc));
        match res {
            VisitResult::Continue => {
                stop_visit |= visit_type_declaration(td, &td.name, &td.loc, visitor).is_err()
            }
            VisitResult::Skip => continue,
            VisitResult::Stop => stop_visit = true,
        }
        if stop_visit {
            return;
        }
    }
}

type Stop = std::result::Result<(), ()>;

const STOP: Stop = Stop::Err(());
const CONTINUE: Stop = Stop::Ok(());

fn get_keyword<'a>(keyword: &'static str, line: u32) -> Visit<'a> {
    let start = Position::new(line, 0);
    let mut end = start;
    end.character += keyword.len() as u32;
    let loc = Loc::new(start, end);

    Visit::Keyword(keyword, loc)
}

fn visit_type_declaration<'a, V: Visitor<'a>>(
    td: &'a TypeDeclaration<Loc, Str>,
    type_name: &'a Str,
    type_loc: &'a Loc,
    visitor: &mut V,
) -> Stop {
    for d in td.dependencies.iter() {
        let res = visitor.visit(Visit::Dependency(&d.name, &d.loc));
        match res {
            VisitResult::Continue => visit_type_expression(d, visitor)?,
            VisitResult::Skip => continue,
            VisitResult::Stop => return STOP,
        }
    }

    match &td.body {
        TypeDefinition::Message(fields) => {
            let res = visitor.visit(Visit::message_constructor(type_name, type_loc));
            match res {
                VisitResult::Continue => visit_constructor(fields, visitor)?,
                VisitResult::Skip => return CONTINUE,
                VisitResult::Stop => return STOP,
            }
        }
        TypeDefinition::Enum(enum_branchs) => {
            for branch in enum_branchs.iter() {
                let res = visitor.visit(Visit::Branch);
                match res {
                    VisitResult::Continue => {}
                    VisitResult::Skip => continue,
                    VisitResult::Stop => return STOP,
                }

                for pattern in branch.patterns.iter() {
                    visit_pattern(pattern, visitor)?;
                }

                for constructor in branch.constructors.iter() {
                    let visit = Visit::enum_constructor(&constructor.name, &constructor.loc);
                    let res = visitor.visit(visit);
                    match res {
                        VisitResult::Continue => {}
                        VisitResult::Skip => continue,
                        VisitResult::Stop => return STOP,
                    }

                    visit_constructor(constructor, visitor)?;
                }
            }
        }
    }

    CONTINUE
}

fn visit_pattern<'a, V: Visitor<'a>>(p: &'a Pattern<Loc, Str>, visitor: &mut V) -> Stop {
    match &p.node {
        PatternNode::Call { name, fields } => {
            if is_correct_type_name(name.as_ref()) {
                let res = visitor.visit(Visit::PatternCall(name, &p.loc));
                match res {
                    VisitResult::Continue => visit_pattern_call_arguments(fields, visitor)?,
                    VisitResult::Skip => return CONTINUE,
                    VisitResult::Stop => return STOP,
                }
            } else {
                assert!(fields.is_empty());

                let res = visitor.visit(Visit::PatternAlias(name));
                match res {
                    VisitResult::Continue => return CONTINUE,
                    VisitResult::Skip => panic!("pattern alias can't be skipped"),
                    VisitResult::Stop => return STOP,
                }
            }
        }
        PatternNode::Literal(l) => visit_pattern_literal(l, &p.loc, visitor)?,
        PatternNode::Underscore => {
            let res = visitor.visit(Visit::PatternUnderscore(&p.loc));
            match res {
                VisitResult::Continue => return CONTINUE,
                VisitResult::Skip => panic!("pattern underscore can't be skipped"),
                VisitResult::Stop => return STOP,
            }
        }
    }

    CONTINUE
}

fn visit_pattern_call_arguments<'a, V: Visitor<'a>>(
    p: &'a [Pattern<Loc, Str>],
    visitor: &mut V,
) -> Stop {
    for p in p.iter() {
        let res = visitor.visit(Visit::PatternCallArgument(None)); // TODO
        match res {
            VisitResult::Continue => {}
            VisitResult::Skip => continue,
            VisitResult::Stop => return STOP,
        }

        visit_pattern(p, visitor)?;
    }

    let res = visitor.visit(Visit::PatternCallStop);
    match res {
        VisitResult::Continue => CONTINUE,
        VisitResult::Skip => panic!("pattern call stop can't be skipped"),
        VisitResult::Stop => STOP,
    }
}

fn visit_pattern_literal<'a, V: Visitor<'a>>(
    l: &'a Literal,
    loc: &'a Loc,
    visitor: &mut V,
) -> Stop {
    let res = visitor.visit(Visit::PatternLiteral(l, loc));
    match res {
        VisitResult::Continue => {}
        VisitResult::Skip => panic!("pattern literal can't be skipped"),
        VisitResult::Stop => return STOP,
    }

    CONTINUE
}

fn visit_constructor<'a, V: Visitor<'a>>(
    c: &'a ConstructorBody<Loc, Str>,
    visitor: &mut V,
) -> Stop {
    for field in c.iter() {
        let res = visitor.visit(Visit::Filed(&field.name, &field.loc));
        match res {
            VisitResult::Continue => {}
            VisitResult::Skip => continue,
            VisitResult::Stop => return STOP,
        }

        visit_type_expression(field, visitor)?;
    }

    CONTINUE
}

fn visit_type_expression<'a, V: Visitor<'a>>(
    te: &'a TypeExpression<Loc, Str>,
    visitor: &mut V,
) -> Stop {
    if let ExpressionNode::FunCall { fun, args } = &te.node {
        let res = visitor.visit(Visit::TypeExpression(fun, &te.loc));
        match res {
            VisitResult::Continue => {}
            VisitResult::Skip => return CONTINUE,
            VisitResult::Stop => return STOP,
        }

        for expr in args.iter() {
            let res = visitor.visit(Visit::Expression(&expr.loc));

            match res {
                VisitResult::Continue => {}
                VisitResult::Skip => continue,
                VisitResult::Stop => return STOP,
            };

            visit_expression(expr, visitor)?;
        }

        return CONTINUE;
    }

    panic!("bad type expression");
}

fn visit_expression<'a, V: Visitor<'a>>(e: &'a Expression<Loc, Str>, visitor: &mut V) -> Stop {
    match &e.node {
        ExpressionNode::OpCall(OpCall::Binary(_, lhs, rhs)) => {
            visit_expression(lhs, visitor)?;
            let (op, loc) = get_operator(e);
            let res = visitor.visit(Visit::Operator(op, loc));
            match res {
                VisitResult::Continue => visit_expression(rhs, visitor)?,
                VisitResult::Skip => panic!("operator can't be skipped"),
                VisitResult::Stop => return STOP,
            }
        }
        ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Access(s), lhs)) => {
            let res = visitor.visit(Visit::AccessChainStart);
            match res {
                VisitResult::Continue => visit_access_chain(lhs, visitor)?,
                VisitResult::Skip => return CONTINUE,
                VisitResult::Stop => return STOP,
            }

            let res = visitor.visit(Visit::AccessChainLast(s));
            match res {
                VisitResult::Continue => return CONTINUE,
                VisitResult::Skip => panic!("access chain last can't be skipped"),
                VisitResult::Stop => return STOP,
            }
        }
        ExpressionNode::OpCall(OpCall::Unary(_, rhs)) => {
            let (op, loc) = get_operator(e);
            let res = visitor.visit(Visit::Operator(op, loc));
            match res {
                VisitResult::Continue => visit_expression(rhs, visitor)?,
                VisitResult::Skip => panic!("operator can't be skipped"),
                VisitResult::Stop => return STOP,
            }
        }
        ExpressionNode::OpCall(OpCall::Literal(l)) => {
            let res = visitor.visit(Visit::Literal(l, &e.loc));
            match res {
                VisitResult::Continue => return CONTINUE,
                VisitResult::Skip => panic!("literal can't be skipped"),
                VisitResult::Stop => return STOP,
            }
        }
        ExpressionNode::FunCall { fun, args } => {
            if fun.as_ref().chars().next().unwrap().is_uppercase() {
                let res = visitor.visit(Visit::ConstructorExpr(fun));
                match res {
                    VisitResult::Continue => {}
                    VisitResult::Skip => return CONTINUE,
                    VisitResult::Stop => return STOP,
                }

                for expr in args.iter() {
                    let res = visitor.visit(Visit::ConstructorExprArgument(None)); // TOOD
                    match res {
                        VisitResult::Continue => {}
                        VisitResult::Skip => continue,
                        VisitResult::Stop => return STOP,
                    }

                    visit_expression(expr, visitor)?;
                }

                let res = visitor.visit(Visit::ConstructorExprStop);
                match res {
                    VisitResult::Continue => return CONTINUE,
                    VisitResult::Skip => panic!("constructor expression stop can't be skipped"),
                    VisitResult::Stop => return STOP,
                }
            } else {
                assert!(fun.as_ref().chars().next().unwrap().is_lowercase());
                assert!(args.is_empty());
                let res = visitor.visit(Visit::VarAccess(fun));
                match res {
                    VisitResult::Continue => return CONTINUE,
                    VisitResult::Skip => panic!("variable access can't be skipped"),
                    VisitResult::Stop => return STOP,
                }
            }
        }
        ExpressionNode::TypedHole => panic!("bad expression: type hole"),
    }

    CONTINUE
}

fn visit_access_chain<'a, V: Visitor<'a>>(e: &'a Expression<Loc, Str>, visitor: &mut V) -> Stop {
    match &e.node {
        ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Access(s), lhs)) => {
            visit_access_chain(lhs, visitor)?;

            let res = visitor.visit(Visit::AccessChain(s));
            match res {
                VisitResult::Continue => {}
                VisitResult::Skip => panic!("access chain can't be skipped"),
                VisitResult::Stop => return STOP,
            }
        }
        ExpressionNode::FunCall { fun, args } => {
            assert!(fun.as_ref().chars().next().unwrap().is_lowercase());
            assert!(args.is_empty());

            let res = visitor.visit(Visit::AccessChain(fun));
            match res {
                VisitResult::Continue => {}
                VisitResult::Skip => panic!("acess chain can't be skiped"),
                VisitResult::Stop => return STOP,
            }
        }
        _ => panic!("bad access chain"),
    };

    let mut loc = Loc::new(e.loc.end, e.loc.end);
    loc.end.character += 1;

    let res = visitor.visit(Visit::AccessDot(loc)); // TODO: better find location
    match res {
        VisitResult::Continue => CONTINUE,
        VisitResult::Skip => panic!("access chain dot can't be skipped"),
        VisitResult::Stop => STOP,
    }
}

// TODO: better find location
fn get_operator(e: &Expression<Loc, Str>) -> (&'static str, Loc) {
    match &e.node {
        ExpressionNode::OpCall(OpCall::Binary(BinaryOp::Plus, lhs, _)) => {
            let start = lhs.loc.end;
            let mut end = start;
            end.character += 1;
            let loc = Loc::new(start, end);
            ("+", loc)
        }
        ExpressionNode::OpCall(OpCall::Binary(BinaryOp::Minus, lhs, _)) => {
            let start = lhs.loc.end;
            let mut end = start;
            end.character += 1;
            let loc = Loc::new(start, end);
            ("-", loc)
        }
        ExpressionNode::OpCall(OpCall::Binary(BinaryOp::Star, lhs, _)) => {
            let start = lhs.loc.end;
            let mut end = start;
            end.character += 1;
            let loc = Loc::new(start, end);
            ("*", loc)
        }
        ExpressionNode::OpCall(OpCall::Binary(BinaryOp::Slash, lhs, _)) => {
            let start = lhs.loc.end;
            let mut end = start;
            end.character += 1;
            let loc = Loc::new(start, end);
            ("/", loc)
        }
        ExpressionNode::OpCall(OpCall::Binary(BinaryOp::And, lhs, _)) => {
            let start = lhs.loc.end;
            let mut end = start;
            end.character += 2;
            let loc = Loc::new(start, end);
            ("&&", loc)
        }
        ExpressionNode::OpCall(OpCall::Binary(BinaryOp::Or, lhs, _)) => {
            let start = lhs.loc.end;
            let mut end = start;
            end.character += 2;
            let loc = Loc::new(start, end);
            ("||", loc)
        }
        ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Bang, expr)) => {
            let end = expr.loc.start;
            let mut start = end;
            start.character -= 1;
            let loc = Loc::new(start, end);
            ("!", loc)
        }
        ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Minus, expr)) => {
            let end = expr.loc.start;
            let mut start = end;
            start.character -= 1;
            let loc = Loc::new(start, end);
            ("-", loc)
        }
        _ => panic!("Unknow operator"),
    }
}
