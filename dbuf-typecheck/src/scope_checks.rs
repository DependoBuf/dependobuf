use std::collections::HashSet;

use dbuf_core::ast::{
    operators::OpCall,
    parsed::{ExpressionNode, PatternNode, TypeDefinition},
};
use thiserror::Error;

use crate::interning::{
    InternedConstructor, InternedExpression, InternedModule, InternedPattern, InternedString,
    InternedTypeDeclaration,
};

#[derive(Default, Debug)]
pub struct ScopeChecker {
    types: HashSet<InternedString>,
    constructors: HashSet<InternedString>,
    signature_scope: HashSet<InternedString>,
    pattern_scope: HashSet<InternedString>,
    constructor_scope: HashSet<InternedString>,
}

#[derive(Error)]
pub enum ScopeCheckerError<Str> {
    #[error("found duplicate type name: `{0}`")]
    DuplicateType(Str),
    #[error("found duplicate constructor name: `{0}`")]
    DuplicateConstructor(Str),
    #[error("found duplicate dependency: `{0}`")]
    DuplicateDependency(Str),
    #[error("found duplicate pattern variable: `{0}`")]
    DuplicatePattern(Str),
    #[error("found duplicate field: `{0}`")]
    DuplicateField(Str),
    #[error("name `{0}` is not in scope")]
    NotInScope(Str),
    #[error("unknown type name: `{0}`")]
    NotAType(Str),
    #[error("unknown constructor name: `{0}`")]
    NotAConstructor(Str),
}

type InternedScopeCheckerError = ScopeCheckerError<InternedString>;

impl ScopeChecker {
    pub fn check_module<Loc>(
        module: &InternedModule<Loc>,
    ) -> Result<(), InternedScopeCheckerError> {
        let mut checker: Self = Default::default();
        checker.add_globals_from_module(module)?;
        for definition in module {
            checker.check_type(definition)?
        }
        Ok(())
    }

    fn check_type<Loc>(
        &mut self,
        definition: &InternedTypeDeclaration<Loc>,
    ) -> Result<(), InternedScopeCheckerError> {
        for dependency in &definition.dependencies {
            self.add_dependency(dependency.name)?
        }
        for dependency in &definition.dependencies {
            self.check_expression(&dependency.data)?
        }
        match &definition.body {
            TypeDefinition::Message(constructor) => self.check_constructor(constructor)?,
            TypeDefinition::Enum(branches) => {
                for branch in branches {
                    for pattern in &branch.patterns {
                        self.check_pattern(pattern)?
                    }
                    for constructor in &branch.constructors {
                        self.check_constructor(constructor)?
                    }
                    self.pattern_scope.clear();
                }
            }
        }
        self.signature_scope.clear();
        Ok(())
    }

    fn check_pattern<Loc>(
        &mut self,
        pattern: &InternedPattern<Loc>,
    ) -> Result<(), InternedScopeCheckerError> {
        use PatternNode::*;

        match &pattern.node {
            Variable { name } => self.add_pattern_variable(*name)?,
            ConstructorCall { fields, .. } => {
                for field in fields {
                    self.check_pattern(&field.data)?
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn check_constructor<Loc>(
        &mut self,
        constructor: &InternedConstructor<Loc>,
    ) -> Result<(), InternedScopeCheckerError> {
        for field in constructor {
            self.add_field(field.name)?
        }
        for field in constructor {
            self.check_expression(&field.data)?
        }
        self.constructor_scope.clear();
        Ok(())
    }

    fn check_expression<Loc>(
        &mut self,
        expression: &InternedExpression<Loc>,
    ) -> Result<(), InternedScopeCheckerError> {
        use ExpressionNode::{ConstructorCall, FunCall, TypedHole, Variable};
        use OpCall::*;

        match &expression.node {
            ExpressionNode::OpCall(op) => match op {
                OpCall::Literal(..) => (),
                Unary(_, exp) => self.check_expression(exp)?,
                Binary(_, lhs, rhs) => {
                    self.check_expression(lhs)?;
                    self.check_expression(rhs)?;
                }
            },
            FunCall { args, fun } => {
                self.lookup_types(*fun)?;
                for arg in args.iter() {
                    self.check_expression(arg)?
                }
            }
            ConstructorCall { fields, name } => {
                self.lookup_constructors(*name)?;
                for field in fields {
                    self.check_expression(&field.data)?
                }
            }
            Variable { name } => self.lookup_scope(*name)?,
            TypedHole => (),
        }
        Ok(())
    }

    fn add_globals_from_module<Loc>(
        &mut self,
        module: &InternedModule<Loc>,
    ) -> Result<(), InternedScopeCheckerError> {
        for declaration in module {
            self.add_type(declaration.name)?;
            match &declaration.body {
                TypeDefinition::Enum(branches) => {
                    for branch in branches {
                        for constructor in &branch.constructors {
                            self.add_constructor(constructor.name)?
                        }
                    }
                }
                TypeDefinition::Message(..) => self.add_constructor(declaration.name)?,
            }
        }
        Ok(())
    }

    fn add_type(&mut self, name: InternedString) -> Result<(), InternedScopeCheckerError> {
        if !self.types.insert(name) {
            Err(ScopeCheckerError::DuplicateType(name))
        } else {
            Ok(())
        }
    }

    fn add_constructor(&mut self, name: InternedString) -> Result<(), InternedScopeCheckerError> {
        if !self.types.insert(name) {
            Err(ScopeCheckerError::DuplicateConstructor(name))
        } else {
            Ok(())
        }
    }

    fn add_dependency(&mut self, name: InternedString) -> Result<(), InternedScopeCheckerError> {
        if !self.signature_scope.insert(name) {
            Err(ScopeCheckerError::DuplicateDependency(name))
        } else {
            Ok(())
        }
    }

    fn add_pattern_variable(
        &mut self,
        name: InternedString,
    ) -> Result<(), InternedScopeCheckerError> {
        if !self.pattern_scope.insert(name) {
            Err(ScopeCheckerError::DuplicatePattern(name))
        } else {
            Ok(())
        }
    }

    fn add_field(&mut self, name: InternedString) -> Result<(), InternedScopeCheckerError> {
        if !self.constructor_scope.insert(name) {
            Err(ScopeCheckerError::DuplicateField(name))
        } else {
            Ok(())
        }
    }

    fn lookup_scope(&self, name: InternedString) -> Result<(), InternedScopeCheckerError> {
        if self.signature_scope.contains(&name)
            || self.pattern_scope.contains(&name)
            || self.constructor_scope.contains(&name)
        {
            Ok(())
        } else {
            Err(ScopeCheckerError::NotInScope(name))
        }
    }

    fn lookup_types(&self, name: InternedString) -> Result<(), InternedScopeCheckerError> {
        if self.types.contains(&name) {
            Ok(())
        } else {
            Err(ScopeCheckerError::NotAType(name))
        }
    }

    fn lookup_constructors(&self, name: InternedString) -> Result<(), InternedScopeCheckerError> {
        if self.constructors.contains(&name) {
            Ok(())
        } else {
            Err(ScopeCheckerError::NotAConstructor(name))
        }
    }
}
