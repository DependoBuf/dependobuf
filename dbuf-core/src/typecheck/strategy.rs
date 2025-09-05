use std::{collections::HashMap, iter::zip};

use crate::ast::{
    operators::{OpCall, UnaryOp},
    parsed::{ExpressionNode, PatternNode, TypeDefinition},
};
use thiserror::Error;

use super::{
    graph::TopSortBuilder,
    interning::{
        InternedConstructor, InternedExpression, InternedModule, InternedPattern, InternedString,
        InternedTypeDeclaration,
    },
    simple::{SimpleTyper, SimpleTyperError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckerTask {
    Signature(InternedString),
    Branch {
        type_name: InternedString,
        branch_index: usize,
    },
    Constuctor(InternedString),
}

#[derive(Debug)]
pub struct StrategyBuilder {
    typer: SimpleTyper,
    signature_scope: HashMap<InternedString, InternedString>,
    pattern_scope: HashMap<InternedString, InternedString>,
    constructor_scope: HashMap<InternedString, InternedString>,
    graph: TopSortBuilder<CheckerTask>,
}

#[derive(Error)]
pub enum StrategyError<Str> {
    #[error(transparent)]
    SimpleTyperError(#[from] SimpleTyperError<Str>),
    #[error("incorrect access operator")]
    IncorrectAccess,
    #[error("number of patterns isn't equal to number of dependencies")]
    IncorrectNumberOfPatterns,
}

type InternedStrategyError = StrategyError<InternedString>;

impl StrategyBuilder {
    pub fn build_strategy<Loc>(
        module: &InternedModule<Loc>,
    ) -> Result<Vec<CheckerTask>, InternedStrategyError> {
        let mut builder = Self {
            typer: SimpleTyper::from_module(module)?,
            signature_scope: Default::default(),
            pattern_scope: Default::default(),
            constructor_scope: Default::default(),
            graph: TopSortBuilder::new(),
        };
        for definition in module {
            builder.analyze_type(definition)?;
        }
        Ok(builder.graph.top_sort())
    }

    fn analyze_type<Loc>(
        &mut self,
        definition: &InternedTypeDeclaration<Loc>,
    ) -> Result<(), InternedStrategyError> {
        let dependencies: Vec<_> = definition
            .dependencies
            .iter()
            .map(|dependency| {
                let ExpressionNode::FunCall { fun: type_name, .. } = dependency.node else {
                    unreachable!()
                };
                self.signature_scope.insert(dependency.name, type_name);
                (dependency.name, type_name)
            })
            .collect();
        for dependency in &definition.dependencies {
            self.analyze_expression(&dependency.data, CheckerTask::Signature(definition.name))?;
        }
        let signature_target = CheckerTask::Signature(definition.name);
        match &definition.body {
            TypeDefinition::Message(constructor) => {
                let constructor_target = CheckerTask::Constuctor(definition.name);
                self.graph.add_edge(&constructor_target, &signature_target);
                self.analyze_constructor(constructor, constructor_target)?;
            }
            TypeDefinition::Enum(branches) => {
                for (branch_index, branch) in branches.iter().enumerate() {
                    let branch_target = CheckerTask::Branch {
                        type_name: definition.name,
                        branch_index: branch_index,
                    };
                    self.graph.add_edge(&branch_target, &signature_target);
                    if branch.patterns.len() != dependencies.len() {
                        return Err(StrategyError::IncorrectNumberOfPatterns);
                    }
                    for (pattern, (_, type_name)) in zip(&branch.patterns, &dependencies) {
                        self.analyze_pattern(
                            pattern,
                            CheckerTask::Branch {
                                type_name: definition.name,
                                branch_index: branch_index,
                            },
                            *type_name,
                        )?;
                    }
                    for constructor in &branch.constructors {
                        let constructor_target = CheckerTask::Constuctor(constructor.name);
                        self.graph.add_edge(&constructor_target, &branch_target);
                        self.analyze_constructor(constructor, constructor_target)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn analyze_expression<Loc>(
        &mut self,
        expression: &InternedExpression<Loc>,
        target: CheckerTask,
    ) -> Result<Option<InternedString>, InternedStrategyError> {
        use ExpressionNode::{ConstructorCall, FunCall, TypedHole, Variable};
        use OpCall::*;
        use UnaryOp::*;

        let type_name = match &expression.node {
            ExpressionNode::OpCall(op) => match op {
                Literal(..) => None,
                Unary(op, exp) => match op {
                    Access(field) => {
                        let lhs_type = self
                            .analyze_expression(exp, target)?
                            .ok_or(StrategyError::IncorrectAccess)?;
                        let type_name = self.typer.get_field(lhs_type, *field)?;
                        self.graph
                            .add_edge(&target, &CheckerTask::Constuctor(lhs_type));
                        Some(type_name)
                    }
                    _ => None,
                },
                Binary(_, lhs, rhs) => {
                    self.analyze_expression(lhs, target)?;
                    self.analyze_expression(rhs, target)?;
                    None
                }
            },
            FunCall { fun, args } => {
                self.graph.add_edge(&target, &CheckerTask::Signature(*fun));
                for arg in args.iter() {
                    self.analyze_expression(arg, target)?;
                }
                None
            }
            ConstructorCall { name, fields } => {
                self.graph
                    .add_edge(&target, &CheckerTask::Constuctor(*name));
                for field in fields {
                    self.analyze_expression(&field, target)?;
                }
                None
            }
            Variable { name } => self.lookup_scope(*name),
            TypedHole => None,
        };
        Ok(type_name)
    }

    fn analyze_pattern<Loc>(
        &mut self,
        pattern: &InternedPattern<Loc>,
        target: CheckerTask,
        current_type: InternedString,
    ) -> Result<(), InternedStrategyError> {
        use PatternNode::*;
        match &pattern.node {
            ConstructorCall { name, fields } => {
                self.graph
                    .add_edge(&target, &CheckerTask::Constuctor(*name));
                for field in fields {
                    let type_name = self.typer.get_field_from_constructor(*name, field.name)?;
                    self.analyze_pattern(&field, target, type_name)?;
                }
            }
            Variable { name } => {
                self.pattern_scope.insert(*name, current_type);
            }
            _ => (),
        }
        Ok(())
    }

    fn analyze_constructor<Loc>(
        &mut self,
        constructor: &InternedConstructor<Loc>,
        target: CheckerTask,
    ) -> Result<(), InternedStrategyError> {
        for field in constructor {
            let ExpressionNode::FunCall { fun: type_name, .. } = field.node else {
                unreachable!()
            };
            self.constructor_scope.insert(field.name, type_name);
        }
        for field in constructor {
            self.analyze_expression(&field.data, target)?;
        }
        Ok(())
    }

    fn lookup_scope(&self, name: InternedString) -> Option<InternedString> {
        if let Some(constructor_name) = self.constructor_scope.get(&name) {
            return Some(*constructor_name);
        }
        if let Some(constructor_name) = self.pattern_scope.get(&name) {
            return Some(*constructor_name);
        }
        if let Some(constructor_name) = self.signature_scope.get(&name) {
            return Some(*constructor_name);
        }
        None
    }
}
