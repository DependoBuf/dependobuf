//! Provides utilities for pretty-printing AST (Abstract Syntax Tree) representations.
//!
//! This module contains tools to:
//! - Format AST nodes as human-readable text
//!

use std::fmt::{Result, Write};

use dbuf_core::ast::operators::*;
use dbuf_core::ast::parsed::definition::*;
use dbuf_core::ast::parsed::*;

use super::ast_access::{Loc, ParsedAst, Position, Str};

// TODO:
//   * use pretty lib (?)
//   * make configuration
//   * TODO resolutions in code

/// A configurable AST pretty-printer.
pub struct PrettyPrinter<'a, W: Write> {
    cursor: Position,
    writer: &'a mut W,
    tab_size: u32,
    header_only: bool,
    with_dependencies: bool,
}

impl<'a, W: Write> PrettyPrinter<'a, W> {
    const MESSAGE_TEXT: &'a str = "message ";
    const ENUM_TEXT: &'a str = "enum ";

    pub fn new(writer: &'a mut W) -> PrettyPrinter<'a, W> {
        PrettyPrinter {
            cursor: Position::new(0, 0),
            writer,
            tab_size: 4,
            header_only: false,
            with_dependencies: true,
        }
    }

    pub fn with_tab_size(mut self, tab_size: u32) -> Self {
        self.tab_size = tab_size;
        self
    }

    pub fn with_header_only(mut self) -> Self {
        self.header_only = true;
        self
    }

    pub fn without_dependencies(mut self) -> Self {
        self.with_dependencies = false;
        self
    }

    fn new_line(&mut self) -> Result {
        self.cursor.line += 1;
        self.cursor.character = 0;
        writeln!(self.writer)?;
        Ok(())
    }

    fn new_line_if(&mut self, predicate: bool) -> Result {
        if predicate {
            self.new_line()?;
        }
        Ok(())
    }

    fn write(&mut self, s: impl AsRef<str>) -> Result {
        let r = s.as_ref();
        self.cursor.character += r.len() as u32;
        write!(self.writer, "{}", r)?;
        Ok(())
    }

    fn write_if(&mut self, predicate: bool, s: impl AsRef<str>) -> Result {
        if predicate {
            self.write(s)?;
        }
        Ok(())
    }

    fn write_tabs(&mut self, tab_count: u32) -> Result {
        let spaces = self.tab_size * tab_count;
        self.cursor.character += spaces;
        let to_write = " ".repeat(spaces as usize);
        write!(self.writer, "{}", to_write)?;
        Ok(())
    }

    pub fn print_ast(&mut self, ast: &ParsedAst) -> Result {
        let mut first = true;

        for definition in ast.iter() {
            if !first {
                self.new_line()?;
                self.new_line()?;
            }
            self.print_type_definition(definition)?;
            first = false;
        }

        Ok(())
    }

    pub fn print_type(&mut self, ast: &ParsedAst, type_name: &str) -> Result {
        let t = ast.iter().find(|d| d.name.as_ref() == type_name);
        if let Some(td) = t {
            self.print_type_definition(td)?;
        }
        Ok(())
    }

    pub fn print_selected_dependency(
        &mut self,
        ast: &ParsedAst,
        type_name: &str,
        dependency: &str,
    ) -> Result {
        let d = ast
            .iter()
            .find(|d| d.name.as_ref() == type_name)
            .map(|d| &d.data.dependencies);
        if let Some(dependencies) = d {
            let d = dependencies.iter().find(|d| d.name.as_ref() == dependency);
            if let Some(dep) = d {
                self.write(&dep.name)?;
                self.write(" ")?;
                self.print_type_expression(&dep.data)?;
            }
        }
        Ok(())
    }

    pub fn print_selected_field(
        &mut self,
        ast: &ParsedAst,
        type_name: &str,
        constructor: &str,
        field: &str,
    ) -> Result {
        let d = ast
            .iter()
            .find(|d| d.name.as_ref() == type_name)
            .map(|d| &d.data.body);

        match d {
            Some(TypeDefinition::Message(m)) => {
                let f = m.iter().find(|f| f.name.as_ref() == field);
                if let Some(field) = f {
                    self.write(&field.name)?;
                    self.write(" ")?;
                    self.print_type_expression(&field.data)?;
                }
            }
            Some(TypeDefinition::Enum(e)) => {
                for b in e.iter() {
                    for ct in b.constructors.iter() {
                        if ct.name.as_ref() == constructor {
                            let f = ct.iter().find(|f| f.name.as_ref() == field);
                            if let Some(field) = f {
                                self.write(&field.name)?;
                                self.write(" ")?;
                                self.print_type_expression(&field.data)?;
                            }
                            return Ok(());
                        }
                    }
                }
            }
            None => {}
        }

        Ok(())
    }

    fn print_type_definition(
        &mut self,
        definition: &Definition<Loc, Str, TypeDeclaration<Loc, Str>>,
    ) -> Result {
        match definition.data.body {
            TypeDefinition::Message(_) => {
                self.write(Self::MESSAGE_TEXT)?;
                self.write(&definition.name)?;
            }
            TypeDefinition::Enum(_) => {
                self.write(Self::ENUM_TEXT)?;
                self.write(&definition.name)?;
            }
        }

        self.write_if(!self.header_only || self.with_dependencies, " ")?;
        self.print_type_declaration(&definition.data)?;

        Ok(())
    }

    fn print_type_declaration(&mut self, type_declaration: &TypeDeclaration<Loc, Str>) -> Result {
        if self.with_dependencies {
            for dependency in type_declaration.dependencies.iter() {
                self.print_dependency(dependency)?;
                self.write(" ")?;
            }
        }

        if !self.header_only {
            self.write("{")?;

            match &type_declaration.body {
                TypeDefinition::Message(constructor) => {
                    self.new_line_if(!constructor.is_empty())?;
                    self.print_constructor(constructor, 1)?;
                }
                TypeDefinition::Enum(branches) => {
                    for branch in branches.iter() {
                        self.new_line()?;
                        self.print_enum_bracnh(branch)?;
                    }
                }
            }

            self.new_line()?;
            self.write("}")?;
        }
        Ok(())
    }

    fn print_dependency(
        &mut self,
        dependency: &Definition<Loc, Str, TypeExpression<Loc, Str>>,
    ) -> Result {
        self.write("(")?;
        self.write(&dependency.name)?;
        self.write(" ")?;
        self.print_type_expression(&dependency.data)?;
        self.write(")")?;

        Ok(())
    }

    fn print_enum_bracnh(&mut self, branch: &EnumBranch<Loc, Str>) -> Result {
        self.write_tabs(1)?;

        let mut first = true;
        for p in branch.patterns.iter() {
            if !first {
                self.write(", ")?;
            }
            self.print_pattern(p)?;
            first = false;
        }

        self.write(" => {")?;

        for c in branch.constructors.iter() {
            self.new_line()?;
            self.print_enum_constructor(c)?;
        }

        self.new_line()?;
        self.write_tabs(1)?;
        self.write("}")?;
        Ok(())
    }

    fn print_pattern(&mut self, pattern: &Pattern<Loc, Str>) -> Result {
        match &pattern.node {
            PatternNode::Call { name, fields } => {
                if fields.is_empty() {
                    // Assuming: variable
                    self.write(name)?;
                } else {
                    // Assuming: constructor
                    self.write(name)?;
                    self.write("{")?;
                    // TODO: parse constructor
                    self.write("}")?;
                }
            }
            PatternNode::Literal(literal) => {
                self.print_literal(literal)?;
            }
            PatternNode::Underscore => {
                self.write("*")?;
            }
        }

        Ok(())
    }

    fn print_enum_constructor(
        &mut self,
        constructor: &Definition<Loc, Str, ConstructorBody<Loc, Str>>,
    ) -> Result {
        self.write_tabs(2)?;

        self.write(&constructor.name)?;
        self.write(" {")?;
        self.new_line_if(!constructor.data.is_empty())?;
        self.print_constructor(&constructor.data, 3)?;
        self.new_line()?;
        self.write_tabs(2)?;
        self.write("}")?;

        Ok(())
    }

    fn print_constructor(
        &mut self,
        constructor: &ConstructorBody<Loc, Str>,
        offset: u32,
    ) -> Result {
        let mut first = true;
        for definition in constructor.iter() {
            if !first {
                self.new_line()?;
            }
            self.write_tabs(offset)?;

            self.write(&definition.name)?;
            self.write(" ")?;
            self.print_type_expression(&definition.data)?;
            self.write(";")?;

            first = false;
        }
        Ok(())
    }

    fn print_type_expression(&mut self, type_expression: &TypeExpression<Loc, Str>) -> Result {
        match &type_expression.node {
            ExpressionNode::FunCall { fun, args } => {
                self.write(fun)?;

                for expr in args.iter() {
                    self.write(" ")?;
                    self.print_expression(expr)?;
                }
            }
            _ => {
                panic!(
                    "bad type expression at (line {}, cell {})",
                    self.cursor.line, self.cursor.character
                );
            }
        }

        Ok(())
    }

    // TODO: change logic: distinguish variable from empty constructor
    fn print_expression(&mut self, expression: &Expression<Loc, Str>) -> Result {
        match &expression.node {
            ExpressionNode::FunCall { fun, args } => {
                if args.is_empty() {
                    // Assuming: variable cal
                    self.write(fun)?;
                } else {
                    // Assuming: constructor
                    self.write(fun)?;
                    self.write("{")?;
                    // TODO: parse constructor
                    self.write("}")?;
                }
            }
            ExpressionNode::OpCall(op) => {
                self.print_opcall(op)?;
            }
            ExpressionNode::TypedHole => {
                panic!(
                    "bad expression: Typed Hole at (line {}, cell {})",
                    self.cursor.line, self.cursor.character
                )
            }
        };
        Ok(())
    }

    fn print_opcall(&mut self, operation: &OpCall<Str, Rec<Expression<Loc, Str>>>) -> Result {
        match operation {
            OpCall::Literal(literal) => {
                self.print_literal(literal)?;
            }
            OpCall::Unary(op, expr) => {
                self.print_unary(op, expr)?;
            }
            OpCall::Binary(op, expr_left, expr_right) => {
                self.write("(")?;

                self.print_expression(expr_left)?;

                self.write(" ")?;
                self.print_binary(op)?;
                self.write(" ")?;

                self.print_expression(expr_right)?;

                self.write(")")?;
            }
        }
        Ok(())
    }

    fn print_literal(&mut self, literal: &Literal) -> Result {
        match literal {
            Literal::Bool(b) => {
                if *b {
                    self.write("true")?;
                } else {
                    self.write("false")?;
                }
            }
            Literal::Double(d) => {
                self.write(d.to_string())?;
            }
            Literal::Int(i) => {
                self.write(i.to_string())?;
            }
            Literal::Str(s) => {
                self.write("\"")?;
                self.write(s)?;
                self.write("\"")?;
            }
            Literal::UInt(ui) => {
                self.write(ui.to_string())?;
                self.write("u")?;
            }
        }
        Ok(())
    }

    fn print_unary(&mut self, op: &UnaryOp<Str>, expr: &Rec<Expression<Loc, Str>>) -> Result {
        match op {
            UnaryOp::Access(field) => {
                self.print_expression(expr)?;

                self.write(".")?;
                self.write(field)?;
            }
            UnaryOp::Minus => {
                self.write("-(")?;

                self.print_expression(expr)?;

                self.write(")")?;
            }
            UnaryOp::Bang => {
                self.write("!(")?;

                self.print_expression(expr)?;

                self.write(")")?;
            }
        }
        Ok(())
    }

    fn print_binary(&mut self, op: &BinaryOp) -> Result {
        match op {
            BinaryOp::And => {
                self.write("&&")?;
            }
            BinaryOp::Minus => {
                self.write("-")?;
            }
            BinaryOp::Or => {
                self.write("||")?;
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
