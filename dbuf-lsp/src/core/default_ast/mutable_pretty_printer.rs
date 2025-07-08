//! Used in `default_ast` to setup locaions
//!
//! This module contains tools to:
//! - Format AST nodes as human-readable text
//! - Track and update source locations during printing
//!

use std::fmt::{Result, Write};

use dbuf_core::ast::operators::*;
use dbuf_core::ast::parsed::definition::*;
use dbuf_core::ast::parsed::*;

use crate::core::ast_access::{
    Loc, LocStringHelper, LocationHelpers, Position, PositionHelpers, Str,
};

#[derive(Debug, Copy, Clone)]
struct Pos {
    line: usize,
    column: usize,
}

impl Pos {
    fn new(line: usize, column: usize) -> Pos {
        Pos { line, column }
    }
}

impl From<Pos> for Position {
    fn from(value: Pos) -> Self {
        Position::new(value.line, value.column)
    }
}

// TODO:
//   * find a way to easily clone from crate::common::pretty_printer

/// A configurable AST pretty-printer that preserves source location information.
///
pub struct PrettyPrinter<'a, W: Write> {
    cursor: Pos,
    writer: &'a mut W,
}

impl<'a, W: Write> PrettyPrinter<'a, W> {
    const MESSAGE_TEXT: &'a str = "message ";
    const ENUM_TEXT: &'a str = "enum ";

    pub fn new(writer: &'a mut W) -> PrettyPrinter<'a, W> {
        PrettyPrinter {
            cursor: Pos::new(0, 0),
            writer,
        }
    }

    fn new_line(&mut self) -> Result {
        self.cursor.line += 1;
        self.cursor.column = 0;
        writeln!(self.writer)?;
        Ok(())
    }

    fn new_line_if(&mut self, predicate: bool) -> Result {
        if predicate {
            self.new_line()?;
        }
        Ok(())
    }

    fn write(&mut self, s: &str) -> Result {
        self.cursor.column += s.len();
        write!(self.writer, "{s}")?;
        Ok(())
    }

    fn write_str(&mut self, s: &mut Str) -> Result {
        s.set_location_start(self.cursor.into());
        self.cursor.column += s.len();
        write!(self.writer, "{s}")?;
        s.set_location_end(self.cursor.into());
        Ok(())
    }

    fn write_tab(&mut self, len: usize) -> Result {
        self.cursor.column += len;
        let to_write = " ".repeat(len);
        write!(self.writer, "{to_write}")?;
        Ok(())
    }

    pub fn parse_module(&mut self, module: &mut Module<Loc, Str>) -> Result {
        self.cursor = Pos::new(0, 0);
        let mut first = true;

        for definition in module.iter_mut() {
            if !first {
                self.new_line()?;
                self.new_line()?;
            }
            self.parse_type_definition(definition)?;
            first = false;
        }

        Ok(())
    }

    fn parse_type_definition(
        &mut self,
        definition: &mut Definition<Loc, Str, TypeDeclaration<Loc, Str>>,
    ) -> Result {
        definition.loc.reset_start(self.cursor.into());

        match definition.data.body {
            TypeDefinition::Message(_) => {
                self.write(Self::MESSAGE_TEXT)?;
                self.write_str(&mut definition.name)?;
                self.write(" ")?;
            }
            TypeDefinition::Enum(_) => {
                self.write(Self::ENUM_TEXT)?;
                self.write_str(&mut definition.name)?;
                self.write(" ")?;
            }
        }

        self.parse_type_declaration(&mut definition.data)?;

        definition.loc.set_end(self.cursor.into());
        Ok(())
    }

    fn parse_type_declaration(
        &mut self,
        type_declaration: &mut TypeDeclaration<Loc, Str>,
    ) -> Result {
        for dependency in &mut type_declaration.dependencies {
            self.parse_dependency(dependency)?;
            self.write(" ")?;
        }

        self.write("{")?;

        match &mut type_declaration.body {
            TypeDefinition::Message(constructor) => {
                self.new_line_if(!constructor.is_empty())?;
                self.parse_constructor(constructor, 4)?;
            }
            TypeDefinition::Enum(branches) => {
                for branch in branches.iter_mut() {
                    self.new_line()?;
                    self.parse_enum_bracnh(branch)?;
                }
            }
        }

        self.new_line()?;
        self.write("}")?;
        Ok(())
    }

    fn parse_dependency(
        &mut self,
        dependency: &mut Definition<Loc, Str, TypeExpression<Loc, Str>>,
    ) -> Result {
        dependency.loc.reset_start(self.cursor.into());

        self.write("(")?;
        self.write_str(&mut dependency.name)?;
        self.write(" ")?;
        self.parse_type_expression(&mut dependency.data)?;
        self.write(")")?;

        dependency.loc.set_end(self.cursor.into());
        Ok(())
    }

    fn parse_enum_bracnh(&mut self, branch: &mut EnumBranch<Loc, Str>) -> Result {
        self.write_tab(4)?;

        let mut first = true;
        for p in &mut branch.patterns {
            if !first {
                self.write(", ")?;
            }
            self.parse_pattern(p)?;
            first = false;
        }

        self.write(" => {")?;

        for c in &mut branch.constructors {
            self.new_line()?;
            self.parse_enum_constructor(c)?;
        }

        self.new_line()?;
        self.write_tab(4)?;
        self.write("}")?;
        Ok(())
    }

    fn parse_pattern(&mut self, pattern: &mut Pattern<Loc, Str>) -> Result {
        pattern.loc.reset_start(self.cursor.into());

        match &mut pattern.node {
            PatternNode::ConstructorCall { name, fields } => {
                self.write_str(name)?;
                self.write("{")?;

                let mut first = true;
                for f in fields.iter_mut() {
                    f.loc.reset_start(self.cursor.into());
                    if !first {
                        self.write(", ")?;
                    }
                    self.write_str(&mut f.name)?;
                    self.write(": ")?;
                    self.parse_pattern(&mut f.data)?;
                    first = false;
                    f.loc.set_end(self.cursor.into());
                }

                self.write("}")?;
            }
            PatternNode::Variable { name } => {
                self.write_str(name)?;
            }
            PatternNode::Literal(literal) => {
                self.parse_literal(literal)?;
            }
            PatternNode::Underscore => {
                self.write("*")?;
            }
        }

        pattern.loc.set_end(self.cursor.into());
        Ok(())
    }

    fn parse_enum_constructor(
        &mut self,
        constructor: &mut Definition<Loc, Str, ConstructorBody<Loc, Str>>,
    ) -> Result {
        self.write_tab(8)?;
        constructor.loc.reset_start(self.cursor.into());

        self.write_str(&mut constructor.name)?;
        self.write(" {")?;
        self.new_line_if(!constructor.data.is_empty())?;
        self.parse_constructor(&mut constructor.data, 12)?;
        self.new_line()?;
        self.write_tab(8)?;
        self.write("}")?;

        constructor.loc.set_end(self.cursor.into());
        Ok(())
    }

    fn parse_constructor(
        &mut self,
        constructor: &mut ConstructorBody<Loc, Str>,
        offset: u32,
    ) -> Result {
        let mut first = true;
        for definition in constructor.iter_mut() {
            if !first {
                self.new_line()?;
            }
            self.write_tab(offset as usize)?;
            definition.loc.reset_start(self.cursor.into());
            self.write_str(&mut definition.name)?;
            self.write(" ")?;
            self.parse_type_expression(&mut definition.data)?;
            self.write(";")?;

            definition.loc.set_end(self.cursor.into());
            first = false;
        }
        Ok(())
    }

    fn parse_type_expression(&mut self, type_expression: &mut TypeExpression<Loc, Str>) -> Result {
        type_expression.loc.reset_start(self.cursor.into());

        match &mut type_expression.node {
            ExpressionNode::FunCall { fun, args } => {
                self.write_str(fun)?;

                let mut modified = vec![];
                for expr in args.iter() {
                    self.write(" ")?;

                    let mut copy = expr.clone();
                    self.parse_expression(&mut copy)?;
                    modified.push(copy);
                }
                *args = Rec::from(modified);
            }
            _ => {
                panic!(
                    "bad type expression at (line {}, cell {})",
                    self.cursor.line, self.cursor.column
                );
            }
        }

        type_expression.loc.set_end(self.cursor.into());
        Ok(())
    }

    fn parse_expression(&mut self, expression: &mut Expression<Loc, Str>) -> Result {
        expression.loc.reset_start(self.cursor.into());

        match &mut expression.node {
            ExpressionNode::ConstructorCall { name, fields } => {
                self.write_str(name)?;
                self.write("{")?;

                let mut first = true;
                for f in fields.iter_mut() {
                    f.loc.reset_start(self.cursor.into());
                    if !first {
                        self.write(", ")?;
                    }
                    self.write_str(&mut f.name)?;
                    self.write(": ")?;
                    self.parse_expression(&mut f.data)?;
                    first = false;
                    f.loc.set_end(self.cursor.into());
                }

                self.write("}")?;
            }
            ExpressionNode::Variable { name } => {
                self.write_str(name)?;
            }
            ExpressionNode::FunCall { fun: _, args: _ } => {
                panic!("unexpected fun call");
            }
            ExpressionNode::OpCall(op) => {
                self.parse_opcall(op)?;
            }
            ExpressionNode::TypedHole => {
                panic!(
                    "bad expression: Typed Hole at (line {}, cell {})",
                    self.cursor.line, self.cursor.column
                )
            }
        }

        expression.loc.set_end(self.cursor.into());
        Ok(())
    }

    fn parse_opcall(&mut self, operation: &mut OpCall<Str, Rec<Expression<Loc, Str>>>) -> Result {
        match operation {
            OpCall::Literal(literal) => {
                self.parse_literal(literal)?;
            }
            OpCall::Unary(op, expr) => {
                self.parse_unary(op, expr)?;
            }
            OpCall::Binary(op, expr_left, expr_right) => {
                self.write("(")?;

                let mut left = (expr_left.as_ref()).clone();
                self.parse_expression(&mut left)?;
                *expr_left = Rec::new(left);

                self.write(" ")?;
                self.parse_binary(*op)?;
                self.write(" ")?;

                let mut right = (expr_right.as_ref()).clone();
                self.parse_expression(&mut right)?;
                *expr_right = Rec::new(right);

                self.write(")")?;
            }
        }
        Ok(())
    }

    fn parse_literal(&mut self, literal: &mut Literal) -> Result {
        match literal {
            Literal::Bool(b) => {
                if *b {
                    self.write("true")?;
                } else {
                    self.write("false")?;
                }
            }
            Literal::Double(d) => {
                self.write(&d.to_string())?;
            }
            Literal::Int(i) => {
                self.write(&i.to_string())?;
            }
            Literal::Str(s) => {
                self.write("\"")?;
                self.write(s)?;
                self.write("\"")?;
            }
            Literal::UInt(ui) => {
                self.write(&ui.to_string())?;
                self.write("u")?;
            }
        }
        Ok(())
    }

    fn parse_unary(
        &mut self,
        op: &mut UnaryOp<Str>,
        expr: &mut Rec<Expression<Loc, Str>>,
    ) -> Result {
        match op {
            UnaryOp::Access(field) => {
                let mut copy = (expr.as_ref()).clone();
                self.parse_expression(&mut copy)?;
                *expr = Rec::new(copy);

                self.write(".")?;
                self.write_str(field)?;
            }
            UnaryOp::Minus => {
                self.write("-(")?;

                let mut copy = (expr.as_ref()).clone();
                self.parse_expression(&mut copy)?;
                *expr = Rec::new(copy);

                self.write(")")?;
            }
            UnaryOp::Bang => {
                self.write("!(")?;

                let mut copy = (expr.as_ref()).clone();
                self.parse_expression(&mut copy)?;
                *expr = Rec::new(copy);

                self.write(")")?;
            }
        }
        Ok(())
    }

    fn parse_binary(&mut self, op: BinaryOp) -> Result {
        match op {
            BinaryOp::BinaryAnd => {
                self.write("&")?;
            }
            BinaryOp::Minus => {
                self.write("-")?;
            }
            BinaryOp::BinaryOr => {
                self.write("|")?;
            }
            BinaryOp::Plus => {
                self.write("+")?;
            }
            BinaryOp::Slash => {
                self.write("/")?;
            }
            BinaryOp::Star => {
                self.write("*")?;
            }
        }
        Ok(())
    }
}
