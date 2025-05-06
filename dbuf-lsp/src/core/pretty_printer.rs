//! Provides utilities for pretty-printing AST (Abstract Syntax Tree) representations.
//!
//! This module contains tools to:
//! - Format AST nodes as human-readable text
//!

use std::fmt::Write;

use dbuf_core::ast::operators::*;
use dbuf_core::ast::parsed::definition::*;
use dbuf_core::ast::parsed::*;

use super::ast_access::{Loc, ParsedAst, Str};

struct Position {
    line: usize,
    column: usize,
}

impl Position {
    fn new(line: usize, column: usize) -> Position {
        Position { line, column }
    }
}

/// A configurable AST pretty-printer.
///
/// TODO:
/// * use pretty lib (?)
/// * make configuration
/// * TODO resolutions in code
/// * rewrite using ast_visitor?
///   * Pros: easier to write selected
///   * Cons: harder to put symbols correctly (bad whitespaces might occur). Fixes with pretty lib.
pub struct PrettyPrinter<'a, W: Write> {
    cursor: Position,
    writer: &'a mut W,
    tab_size: usize,
    header_only: bool,
    with_dependencies: bool,
}

impl<'a, W: Write> PrettyPrinter<'a, W> {
    const MESSAGE_TEXT: &'a str = "message ";
    const ENUM_TEXT: &'a str = "enum ";

    /// Creates new printer for `writer` with default parameters.
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            cursor: Position::new(0, 0),
            writer,
            tab_size: 4,
            header_only: false,
            with_dependencies: true,
        }
    }

    /// Sets tab size for printer.
    pub fn with_tab_size(mut self, tab_size: usize) -> Self {
        self.tab_size = tab_size;
        self
    }

    fn new_line(&mut self) {
        self.cursor.line += 1;
        self.cursor.column = 0;
        writeln!(self.writer).expect("writeln! considered to be infallible");
    }

    fn new_line_if(&mut self, predicate: bool) {
        if predicate {
            self.new_line();
        }
    }

    fn write(&mut self, s: impl AsRef<str>) {
        let r = s.as_ref();
        self.cursor.column += r.len();
        write!(self.writer, "{}", r).expect("write! considered to be infallible");
    }

    fn write_if(&mut self, predicate: bool, s: impl AsRef<str>) {
        if predicate {
            self.write(s);
        }
    }

    fn write_tabs(&mut self, tab_count: usize) {
        let spaces = self.tab_size * tab_count;
        self.cursor.column += spaces;
        let to_write = " ".repeat(spaces);
        write!(self.writer, "{}", to_write).expect("write! considered to be infallible");
    }

    /// Prints whole ast.
    pub fn print_ast(&mut self, ast: &ParsedAst) {
        let mut first = true;

        for definition in ast.iter() {
            if !first {
                self.new_line();
                self.new_line();
            }
            self.print_type_definition(definition);
            first = false;
        }
    }

    /// Prints type of ast.
    pub fn print_type(&mut self, ast: &ParsedAst, type_name: &str) {
        let t = ast.iter().find(|d| d.name.as_ref() == type_name);
        if let Some(td) = t {
            self.print_type_definition(td);
        }
    }

    /// Prints constructor of ast.
    pub fn print_selected_constructor(
        &mut self,
        ast: &ParsedAst,
        type_name: &str,
        consructor: &str,
    ) {
        let t = ast
            .iter()
            .find(|d| d.name.as_ref() == type_name)
            .map(|d| &d.body);

        if let Some(TypeDefinition::Enum(e)) = t {
            for b in e {
                for c in b.constructors.iter() {
                    if c.name.as_ref() != consructor {
                        continue;
                    }
                    self.write(&c.name);
                    self.write(" {");
                    self.new_line_if(!c.data.is_empty());
                    self.print_constructor(&c.data, 1);
                    self.new_line();
                    self.write("}");
                    return;
                }
            }
        }
    }

    /// Prints selected branch of enum.
    pub fn print_selected_branch(&mut self, ast: &ParsedAst, type_name: &str, branch_id: usize) {
        let t = ast
            .iter()
            .find(|d| d.name.as_ref() == type_name)
            .map(|d| &d.body);

        if let Some(TypeDefinition::Enum(e)) = t {
            let b = e.get(branch_id);
            if let Some(b) = b {
                self.print_all_patterns(&b.patterns);
                self.write("=> {}");
            }
        }
    }

    /// Prints only dependency of type.
    pub fn print_selected_dependency(
        &mut self,
        ast: &ParsedAst,
        type_name: &str,
        dependency: &str,
    ) {
        let d = ast
            .iter()
            .find(|d| d.name.as_ref() == type_name)
            .map(|d| &d.data.dependencies);
        if let Some(dependencies) = d {
            let d = dependencies.iter().find(|d| d.name.as_ref() == dependency);
            if let Some(dep) = d {
                self.write(&dep.name);
                self.write(" ");
                self.print_type_expression(&dep.data);
            }
        }
    }

    /// Prints only field of constructor of type.
    pub fn print_selected_field(
        &mut self,
        ast: &ParsedAst,
        type_name: &str,
        constructor: &str,
        field: &str,
    ) {
        let d = ast
            .iter()
            .find(|d| d.name.as_ref() == type_name)
            .map(|d| &d.data.body);

        match d {
            Some(TypeDefinition::Message(m)) => {
                let f = m.iter().find(|f| f.name.as_ref() == field);
                if let Some(field) = f {
                    self.write(&field.name);
                    self.write(" ");
                    self.print_type_expression(&field.data);
                }
            }
            Some(TypeDefinition::Enum(e)) => {
                for b in e.iter() {
                    for ct in b.constructors.iter() {
                        if ct.name.as_ref() == constructor {
                            let f = ct.iter().find(|f| f.name.as_ref() == field);
                            if let Some(field) = f {
                                self.write(&field.name);
                                self.write(" ");
                                self.print_type_expression(&field.data);
                            }
                            return;
                        }
                    }
                }
            }
            None => {}
        }
    }

    fn print_type_definition(
        &mut self,
        definition: &Definition<Loc, Str, TypeDeclaration<Loc, Str>>,
    ) {
        match definition.data.body {
            TypeDefinition::Message(_) => {
                self.write(Self::MESSAGE_TEXT);
                self.write(&definition.name);
            }
            TypeDefinition::Enum(_) => {
                self.write(Self::ENUM_TEXT);
                self.write(&definition.name);
            }
        }

        self.write_if(!self.header_only || self.with_dependencies, " ");
        self.print_type_declaration(&definition.data);
    }

    fn print_type_declaration(&mut self, type_declaration: &TypeDeclaration<Loc, Str>) {
        if self.with_dependencies {
            for dependency in type_declaration.dependencies.iter() {
                self.print_dependency(dependency);
                self.write(" ");
            }
        }

        if !self.header_only {
            self.write("{");

            match &type_declaration.body {
                TypeDefinition::Message(constructor) => {
                    self.new_line_if(!constructor.is_empty());
                    self.print_constructor(constructor, 1);
                }
                TypeDefinition::Enum(branches) => {
                    for branch in branches.iter() {
                        self.new_line();
                        self.print_enum_bracnh(branch);
                    }
                }
            }

            self.new_line();
            self.write("}");
        }
    }

    fn print_dependency(&mut self, dependency: &Definition<Loc, Str, TypeExpression<Loc, Str>>) {
        self.write("(");
        self.write(&dependency.name);
        self.write(" ");
        self.print_type_expression(&dependency.data);
        self.write(")");
    }

    fn print_enum_bracnh(&mut self, branch: &EnumBranch<Loc, Str>) {
        self.write_tabs(1);

        self.print_all_patterns(&branch.patterns);
        self.write(" => {");

        for c in branch.constructors.iter() {
            self.new_line();
            self.print_enum_constructor(c);
        }

        self.new_line();
        self.write_tabs(1);
        self.write("}");
    }

    fn print_all_patterns(&mut self, patterns: &[Pattern<Loc, Str>]) {
        let mut first = true;
        for p in patterns.iter() {
            if !first {
                self.write(", ");
            }
            self.print_pattern(p);
            first = false;
        }
    }

    fn print_pattern(&mut self, pattern: &Pattern<Loc, Str>) {
        match &pattern.node {
            PatternNode::ConstructorCall { name, fields } => {
                self.write(name);
                self.write("{");
                let mut first = true;
                for f in fields.iter() {
                    if !first {
                        self.write(", ");
                    }
                    self.write(&f.name);
                    self.write(": ");
                    self.print_pattern(&f.data);
                    first = false;
                }
                self.write("}");
            }
            PatternNode::Variable { name } => {
                self.write(name);
            }
            PatternNode::Literal(literal) => {
                self.print_literal(literal);
            }
            PatternNode::Underscore => {
                self.write("*");
            }
        }
    }

    fn print_enum_constructor(
        &mut self,
        constructor: &Definition<Loc, Str, ConstructorBody<Loc, Str>>,
    ) {
        self.write_tabs(2);

        self.write(&constructor.name);
        self.write(" {");
        self.new_line_if(!constructor.data.is_empty());
        self.print_constructor(&constructor.data, 3);
        self.new_line();
        self.write_tabs(2);
        self.write("}");
    }

    fn print_constructor(&mut self, constructor: &ConstructorBody<Loc, Str>, offset: usize) {
        let mut first = true;
        for definition in constructor.iter() {
            if !first {
                self.new_line();
            }
            self.write_tabs(offset);

            self.write(&definition.name);
            self.write(" ");
            self.print_type_expression(&definition.data);
            self.write(";");

            first = false;
        }
    }

    fn print_type_expression(&mut self, type_expression: &TypeExpression<Loc, Str>) {
        match &type_expression.node {
            ExpressionNode::FunCall { fun, args } => {
                self.write(fun);

                for expr in args.iter() {
                    self.write(" ");
                    self.print_expression(expr);
                }
            }
            _ => {
                panic!(
                    "bad type expression at (line {}, cell {})",
                    self.cursor.line, self.cursor.column
                );
            }
        }
    }

    fn print_expression(&mut self, expression: &Expression<Loc, Str>) {
        match &expression.node {
            ExpressionNode::ConstructorCall { name, fields } => {
                self.write(name);
                self.write("{");
                let mut first = true;
                for f in fields.iter() {
                    if !first {
                        self.write(", ");
                    }
                    self.write(&f.name);
                    self.write(": ");
                    self.print_expression(&f.data);
                    first = false;
                }
                self.write("}");
            }
            ExpressionNode::Variable { name } => {
                self.write(name);
            }
            ExpressionNode::FunCall { fun: _, args: _ } => {
                panic!("unexpected TypeExpression: no FunCalls in expression supported");
            }
            ExpressionNode::OpCall(op) => {
                self.print_opcall(op);
            }
            ExpressionNode::TypedHole => {
                panic!(
                    "bad expression: Typed Hole at (line {}, cell {})",
                    self.cursor.line, self.cursor.column
                )
            }
        };
    }

    fn print_opcall(&mut self, operation: &OpCall<Str, Rec<Expression<Loc, Str>>>) {
        match operation {
            OpCall::Literal(literal) => {
                self.print_literal(literal);
            }
            OpCall::Unary(op, expr) => {
                self.print_unary(op, expr);
            }
            OpCall::Binary(op, expr_left, expr_right) => {
                self.write("(");

                self.print_expression(expr_left);

                self.write(" ");
                self.print_binary(op);
                self.write(" ");

                self.print_expression(expr_right);

                self.write(")");
            }
        }
    }

    fn print_literal(&mut self, literal: &Literal) {
        match literal {
            Literal::Bool(b) => {
                if *b {
                    self.write("true");
                } else {
                    self.write("false");
                }
            }
            Literal::Double(d) => {
                self.write(d.to_string());
            }
            Literal::Int(i) => {
                self.write(i.to_string());
            }
            Literal::Str(s) => {
                self.write("\"");
                self.write(s);
                self.write("\"");
            }
            Literal::UInt(ui) => {
                self.write(ui.to_string());
                self.write("u");
            }
        }
    }

    fn print_unary(&mut self, op: &UnaryOp<Str>, expr: &Rec<Expression<Loc, Str>>) {
        match op {
            UnaryOp::Access(field) => {
                self.print_expression(expr);

                self.write(".");
                self.write(field);
            }
            UnaryOp::Minus => {
                self.write("-(");

                self.print_expression(expr);

                self.write(")");
            }
            UnaryOp::Bang => {
                self.write("!(");

                self.print_expression(expr);

                self.write(")");
            }
        }
    }

    fn print_binary(&mut self, op: &BinaryOp) {
        match op {
            BinaryOp::And => {
                self.write("&&");
            }
            BinaryOp::Minus => {
                self.write("-");
            }
            BinaryOp::Or => {
                self.write("||");
            }
            BinaryOp::Plus => {
                self.write("+");
            }
            BinaryOp::Slash => {
                self.write("/");
            }
            BinaryOp::Star => {
                self.write("*");
            }
        }
    }
}
