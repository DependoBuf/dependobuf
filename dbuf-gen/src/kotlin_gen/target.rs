use core::panic;

use pretty::{BoxAllocator, BoxDoc, DocAllocator, DocBuilder};

use crate::{ast, format};

pub struct Field(ast::Symbol);

/// Kotlin's sealed class resembles enum in rust
pub struct SealedClass {
    pub name: String,
    pub fields: Vec<Field>,
    pub constructors: Vec<InnerClass>,
}

/// Inner class of a sealed class resembles enum constructor in rust
pub struct InnerClass {
    pub name: String,
    pub fields: Vec<Field>,
    pub parent_params: Vec<Field>,
    pub result_type: ast::TypeExpression,
}

fn compile_value_expression<'a>(
    alloc: &'a BoxAllocator,
    expr: &ast::ValueExpression,
) -> DocBuilder<'a, BoxAllocator> {
    fn compile_op_call<'a>(
        alloc: &'a BoxAllocator,
        expr: &ast::OpCall,
    ) -> DocBuilder<'a, BoxAllocator> {
        match expr {
            ast::OpCall::Literal(literal) => alloc.text(match literal {
                ast::Literal::Bool(bool) => bool.to_string(),
                ast::Literal::Double(f64) => f64.to_string(),
                ast::Literal::Int(i64) => i64.to_string(),
                ast::Literal::UInt(u64) => u64.to_string(),
                ast::Literal::Str(_string) => panic!("Not implemented"),
            }),

            ast::OpCall::Binary(op, left, right) => {
                let left = compile_value_expression(alloc, left);
                let right = compile_value_expression(alloc, right);

                let op_str = match op {
                    ast::BinaryOp::Plus => alloc.text("+"),
                    ast::BinaryOp::Minus => alloc.text("-"),
                    ast::BinaryOp::Star => alloc.text("*"),
                    ast::BinaryOp::Slash => alloc.text("/"),
                    ast::BinaryOp::BinaryAnd => alloc.text("&"),
                    ast::BinaryOp::BinaryOr => alloc.text("|"),
                };

                left.append(op_str).append(right)
            }
            ast::OpCall::Unary(op, arg) => {
                let arg = compile_value_expression(alloc, arg);

                match op {
                    ast::UnaryOp::Bang => alloc.text("!").append(arg),
                    ast::UnaryOp::Minus => alloc.text("-").append(arg),
                    ast::UnaryOp::Access { to: _, field } => {
                        let field_name = field.upgrade().expect("value to be present").name.clone();

                        arg.append(".").append(field_name)
                    }
                }
            }
        }
    }

    match expr {
        ast::ValueExpression::OpCall(op_call) => compile_op_call(alloc, op_call),
        ast::ValueExpression::Constructor {
            call,
            implicits,
            arguments,
        } => {
            let constructor = call.upgrade().expect("Value to be present");
            let ty = constructor.result_type.get_type();

            let class_name = alloc
                .text(ty.name.clone())
                .append(".")
                .append(constructor.name.clone());

            let parameters = alloc.intersperse(
                arguments
                    .iter()
                    .chain(implicits.iter())
                    .map(|arg| compile_value_expression(alloc, arg)),
                alloc.text(", "),
            );
            class_name.append(parameters.parens())
        }
        ast::ValueExpression::Variable(symbol) => {
            let name = symbol.upgrade().expect("Value to be present").name.clone();
            alloc.text(name)
        }
    }
}

impl SealedClass {
    pub fn generate<'a>(&self, alloc: &'a BoxAllocator) -> BoxDoc<'a> {
        let build_field_declarations = |fields: &Vec<Field>| {
            alloc.concat(fields.iter().map(|field| {
                alloc
                    .text("val")
                    .append(alloc.space())
                    .append(field.generate(alloc))
                    .append(";")
                    .append(alloc.hardline())
            }))
        };

        let build_constructor = |fields: &Vec<Field>| {
            let constructor_params = alloc.intersperse(
                fields.iter().map(|field| field.generate(alloc)),
                alloc.text(", "),
            );

            let assignments = (|| {
                let build_assignment = |name: &String| {
                    alloc
                        .text("this.")
                        .append(name.clone())
                        .append(alloc.space())
                        .append("=")
                        .append(alloc.space())
                        .append(name.clone())
                        .append(";")
                        .append(alloc.hardline())
                };
                alloc.concat(fields.iter().map(|field| build_assignment(&field.0.name)))
            })();

            alloc
                .text("private constructor")
                .append(constructor_params.parens())
                .append(alloc.space())
                .append(
                    alloc
                        .hardline()
                        .append("// constructor asserts")
                        .append(alloc.hardline())
                        .append(assignments)
                        .nest(format::NEST_UNIT)
                        .braces(),
                )
                .append(alloc.hardline())
        };

        let build_inner_classes = |constructors: &Vec<InnerClass>| {
            alloc.concat(
                constructors
                    .iter()
                    .map(|inner_class| inner_class.generate(&self.name, alloc)),
            )
        };

        let build_class_body = |field_declarations, constructor, inner_classes| {
            alloc.concat([field_declarations, constructor, inner_classes])
        };

        let build_class = |name: &String, body| {
            alloc
                .text("sealed class")
                .append(alloc.space())
                .append(name.clone())
                .append(alloc.space())
                .append(
                    alloc
                        .hardline()
                        .append(body)
                        .nest(format::NEST_UNIT)
                        .braces(),
                )
        };

        build_class(
            &self.name,
            build_class_body(
                build_field_declarations(&self.fields),
                build_constructor(&self.fields),
                build_inner_classes(&self.constructors),
            ),
        )
        .into_doc()
    }
}

impl InnerClass {
    pub fn generate<'a>(&self, parent_name: &String, alloc: &'a BoxAllocator) -> BoxDoc<'a> {
        let build_field_declarations = |fields: &Vec<Field>| {
            alloc.concat(fields.iter().map(|field| {
                alloc
                    .text("val")
                    .append(alloc.space())
                    .append(field.generate(alloc))
                    .append(";")
                    .append(alloc.hardline())
            }))
        };

        let build_constructor =
            |fields: &Vec<Field>, parent_params: &Vec<Field>, result_type: &ast::TypeExpression| {
                let constructor_params = alloc.intersperse(
                    parent_params
                        .iter()
                        .chain(fields.iter())
                        .map(|param| param.generate(alloc)),
                    alloc.text(", "),
                );

                let parent_params = match result_type {
                    ast::TypeExpression::Type {
                        call: _,
                        dependencies,
                    } => alloc.intersperse(
                        dependencies
                            .iter()
                            .map(|expr| compile_value_expression(alloc, expr)),
                        alloc.text(", "),
                    ),
                };

                let assignments = (|| {
                    let build_assignment = |name: &String| {
                        alloc
                            .text("this.")
                            .append(name.clone())
                            .append(alloc.space())
                            .append("=")
                            .append(alloc.space())
                            .append(name.clone())
                            .append(";")
                            .append(alloc.hardline())
                    };
                    alloc.concat(fields.iter().map(|field| build_assignment(&field.0.name)))
                })();

                alloc
                    .text("constructor")
                    .append(constructor_params.parens())
                    .append(":")
                    .append(alloc.space())
                    .append("super")
                    .append(parent_params.parens())
                    .append(alloc.space())
                    .append(
                        alloc
                            .hardline()
                            .append("// inner class asserts")
                            .append(alloc.hardline())
                            .append(assignments)
                            .nest(format::NEST_UNIT)
                            .braces(),
                    )
            };

        let build_class_body =
            |field_declarations, constructor| alloc.concat([field_declarations, constructor]);

        let build_class = |name: &String, body| {
            alloc
                .text("class")
                .append(alloc.space())
                .append(name.clone())
                .append(":")
                .append(alloc.space())
                .append(parent_name.clone())
                .append(alloc.space())
                .append(
                    alloc
                        .hardline()
                        .append(body)
                        .nest(format::NEST_UNIT)
                        .append(alloc.hardline())
                        .braces(),
                )
                .append(alloc.hardline())
        };

        build_class(
            &self.name,
            build_class_body(
                build_field_declarations(&self.fields),
                build_constructor(&self.fields, &self.parent_params, &self.result_type),
            ),
        )
        .into_doc()
    }
}

impl Field {
    pub fn new(symbol: &ast::Symbol) -> Self {
        Self(symbol.clone())
    }
    pub fn generate<'a>(&self, alloc: &'a BoxAllocator) -> BoxDoc<'a> {
        alloc
            .text(self.0.name.clone())
            .append(":")
            .append(alloc.space())
            .append(self.0.ty.get_type().name.clone())
            .into_doc()
    }
}
