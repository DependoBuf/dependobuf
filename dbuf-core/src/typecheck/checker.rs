use std::collections::HashMap;

use crate::ast::elaborated::{self, Constructor, ConstructorNames, Module, Type};
use crate::ast::parsed::{self, ExpressionNode, PatternNode, TypeDefinition};
use thiserror::Error;

use super::{
    builtins::BuiltinTypes,
    context::Context,
    interning::{
        InternedExpression, InternedModule, InternedPattern, InternedString, StringInterner,
    },
    simple::{SimpleTyper, SimpleTyperError},
    strategy::{CheckerTask, StrategyBuilder, StrategyError},
};

#[derive(Error, Debug)]
pub enum TypeCheckError {
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        expected: InternedString,
        found: InternedString,
    },

    #[error("undefined type: {0}")]
    UndefinedType(InternedString),

    #[error("undefined constructor: {0}")]
    UndefinedConstructor(InternedString),

    #[error("field access on non-message type: {0}")]
    InvalidFieldAccess(InternedString),

    #[error("missing field in constructor: {field} in {constructor}")]
    MissingField {
        constructor: InternedString,
        field: InternedString,
    },

    #[error("duplicate field in constructor: {field} in {constructor}")]
    DuplicateField {
        constructor: InternedString,
        field: InternedString,
    },

    #[error("cyclic dependency detected in type: {0}")]
    CyclicDependency(InternedString),

    #[error("pattern mismatch: expected {expected}, found {pattern}")]
    PatternMismatch {
        expected: InternedString,
        pattern: InternedString,
    },

    #[error("arity mismatch: expected {expected} arguments, found {found}")]
    ArityMismatch { expected: usize, found: usize },

    #[error("unsupported type expression")]
    UnsupportedTypeExpression,

    #[error(transparent)]
    SimpleTyperError(#[from] SimpleTyperError<InternedString>),

    #[error(transparent)]
    StrategyError(#[from] StrategyError<InternedString>),
}

/// Core type checker that transforms parsed AST into elaborated AST
pub struct TypeChecker {
    /// Context for variable and alias bindings
    context: Box<Context>,
    /// String interner for efficient string handling
    _interner: StringInterner<String>,
    /// Built-in types (String, Int, Bool, etc.)
    builtins: BuiltinTypes,
    /// Simple typer for basic type operations
    simple_typer: SimpleTyper,
    /// Elaborated constructors mapped by name
    constructors: HashMap<InternedString, Constructor<InternedString>>,
    /// Elaborated types mapped by name
    types: HashMap<InternedString, Type<InternedString>>,
}

impl TypeChecker {
    /// Create a new TypeChecker instance
    pub fn new(mut interner: StringInterner<String>) -> Self {
        let builtins = BuiltinTypes::from_interner(&mut interner);

        Self {
            context: Box::new(Context::new()),
            _interner: interner,
            builtins,
            simple_typer: SimpleTyper::default(),
            constructors: HashMap::new(),
            types: HashMap::new(),
        }
    }

    /// Create a new TypeChecker instance with pre-computed builtin types
    /// This should be used when the interner has already been used to intern builtin type names
    pub fn with_builtins(interner: StringInterner<String>, builtins: BuiltinTypes) -> Self {
        Self {
            context: Box::new(Context::new()),
            _interner: interner,
            builtins,
            simple_typer: SimpleTyper::default(),
            constructors: HashMap::new(),
            types: HashMap::new(),
        }
    }

    /// Check and elaborate a complete module
    pub fn check_module<Loc>(
        &mut self,
        module: &InternedModule<Loc>,
    ) -> Result<Module<InternedString>, TypeCheckError> {
        // Build task execution strategy
        let tasks = StrategyBuilder::build_strategy(module)?;

        // Initialize simple typer with module types
        self.simple_typer = SimpleTyper::from_module(module)?;

        // Execute tasks in topologically sorted order
        for task in tasks {
            self.execute_task(task, module)?;
        }

        // Build final elaborated module
        self.build_elaborated_module()
    }

    /// Execute a single type checking task
    fn execute_task<Loc>(
        &mut self,
        task: CheckerTask,
        module: &InternedModule<Loc>,
    ) -> Result<(), TypeCheckError> {
        match task {
            CheckerTask::Signature(type_name) => {
                self.check_signature(type_name, module)?;
            }
            CheckerTask::Branch {
                type_name,
                branch_index,
            } => {
                self.check_branch(type_name, branch_index, module)?;
            }
            CheckerTask::Constructor(constructor_name) => {
                self.check_constructor(constructor_name, module)?;
            }
        }
        Ok(())
    }

    /// Check a type signature and validate its dependencies
    fn check_signature<Loc>(
        &mut self,
        type_name: InternedString,
        module: &InternedModule<Loc>,
    ) -> Result<(), TypeCheckError> {
        // If this is a builtin type, we don't need to process it
        if self.is_builtin_type(type_name) {
            return Ok(());
        }

        // Find the type declaration in the module
        let type_decl = module
            .iter()
            .find(|def| def.name == type_name)
            .ok_or(TypeCheckError::UndefinedType(type_name))?;

        // Create new scope layer for this signature
        self.context = self.context.clone().new_layer();

        // Validate and bind dependency types
        let mut dependencies = Vec::new();
        for dep in &type_decl.data.dependencies {
            let type_expr = self.check_type_expression(&dep.data)?;
            self.context.bind_variable(dep.name, type_expr.clone())?;
            dependencies.push((dep.name, type_expr));
        }

        // Determine constructor names based on type definition
        let constructor_names = match &type_decl.data.body {
            TypeDefinition::Message(_) => ConstructorNames::OfMessage(type_name),
            TypeDefinition::Enum(branches) => {
                let mut names = std::collections::BTreeSet::new();
                for branch in branches {
                    for constructor in &branch.constructors {
                        names.insert(constructor.name);
                    }
                }
                ConstructorNames::OfEnum(names)
            }
        };

        // Store the elaborated type
        let elaborated_type = Type {
            dependencies,
            constructor_names,
        };
        self.types.insert(type_name, elaborated_type);

        // Pop scope layer
        self.context = self.context.clone().pop_layer();

        Ok(())
    }

    /// Check an enum branch and its patterns
    fn check_branch<Loc>(
        &mut self,
        type_name: InternedString,
        branch_index: usize,
        module: &InternedModule<Loc>,
    ) -> Result<(), TypeCheckError> {
        // Find the type declaration and specific branch
        let type_decl = module
            .iter()
            .find(|def| def.name == type_name)
            .ok_or(TypeCheckError::UndefinedType(type_name))?;

        let TypeDefinition::Enum(branches) = &type_decl.data.body else {
            return Err(TypeCheckError::UndefinedType(type_name));
        };

        let branch = branches
            .get(branch_index)
            .ok_or(TypeCheckError::UndefinedType(type_name))?;

        // Create scope layer for pattern variables
        self.context = self.context.clone().new_layer();

        // Validate each pattern against its expected type
        for (i, pattern) in branch.patterns.iter().enumerate() {
            let dep = &type_decl.data.dependencies[i];
            let expected_type = self.check_type_expression(&dep.data)?;
            self.check_pattern(pattern, &expected_type)?;
        }

        // Pop scope layer
        self.context = self.context.clone().pop_layer();

        Ok(())
    }

    /// Check a constructor definition and validate its fields
    fn check_constructor<Loc>(
        &mut self,
        constructor_name: InternedString,
        module: &InternedModule<Loc>,
    ) -> Result<(), TypeCheckError> {
        // Find the constructor definition in the module
        let (constructor_def, parent_type) = self
            .find_constructor_in_module(constructor_name, module)
            .ok_or(TypeCheckError::UndefinedConstructor(constructor_name))?;

        // Create scope layer for constructor fields
        self.context = self.context.clone().new_layer();

        // Get the parent type's dependencies as implicit arguments
        let parent_type_info = self
            .types
            .get(&parent_type)
            .ok_or(TypeCheckError::UndefinedType(parent_type))?
            .clone();
        let implicits = parent_type_info.dependencies.clone();

        // Validate and collect explicit fields
        let mut fields = Vec::new();
        for field_def in constructor_def {
            let field_type = self.check_type_expression(&field_def.data)?;
            self.context
                .bind_variable(field_def.name, field_type.clone())?;
            fields.push((field_def.name, field_type));
        }

        // Create result type
        let result_type = elaborated::TypeExpression::TypeExpression {
            name: parent_type,
            dependencies: std::sync::Arc::new([]),
        };

        // Store elaborated constructor
        let elaborated_constructor = Constructor {
            implicits,
            fields,
            result_type,
        };
        self.constructors
            .insert(constructor_name, elaborated_constructor);

        // Pop scope layer
        self.context = self.context.clone().pop_layer();

        Ok(())
    }

    /// Find a constructor definition within the module
    fn find_constructor_in_module<'a, Loc>(
        &self,
        constructor_name: InternedString,
        module: &'a InternedModule<Loc>,
    ) -> Option<(
        &'a Vec<
            parsed::definition::Definition<
                Loc,
                InternedString,
                parsed::Expression<Loc, InternedString>,
            >,
        >,
        InternedString,
    )> {
        for type_def in module {
            match &type_def.data.body {
                TypeDefinition::Message(constructor) => {
                    if type_def.name == constructor_name {
                        return Some((constructor, type_def.name));
                    }
                }
                TypeDefinition::Enum(branches) => {
                    for branch in branches {
                        for constructor in &branch.constructors {
                            if constructor.name == constructor_name {
                                return Some((&constructor.data, type_def.name));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Check a type expression and return its elaborated form
    fn check_type_expression<Loc>(
        &mut self,
        expr: &InternedExpression<Loc>,
    ) -> Result<elaborated::TypeExpression<InternedString>, TypeCheckError> {
        match &expr.node {
            ExpressionNode::FunCall { fun, args } => {
                // Check if this is a built-in type
                if self.is_builtin_type(*fun) {
                    // Built-in types are always valid
                    let mut dependencies = Vec::new();
                    for arg in args.iter() {
                        let value_expr = self.check_value_expression(arg)?;
                        dependencies.push(value_expr);
                    }

                    Ok(elaborated::TypeExpression::TypeExpression {
                        name: *fun,
                        dependencies: dependencies.into(),
                    })
                } else {
                    // User-defined type - check if it exists
                    if !self.types.contains_key(fun) {
                        return Err(TypeCheckError::UndefinedType(*fun));
                    }

                    // Type application: TypeName(arg1, arg2, ...)
                    let mut dependencies = Vec::new();
                    for arg in args.iter() {
                        let value_expr = self.check_value_expression(arg)?;
                        dependencies.push(value_expr);
                    }

                    Ok(elaborated::TypeExpression::TypeExpression {
                        name: *fun,
                        dependencies: dependencies.into(),
                    })
                }
            }
            ExpressionNode::Variable { name } => {
                // Simple type reference - check if it's built-in or user-defined
                if self.is_builtin_type(*name) || self.types.contains_key(name) {
                    Ok(elaborated::TypeExpression::TypeExpression {
                        name: *name,
                        dependencies: std::sync::Arc::new([]),
                    })
                } else {
                    Err(TypeCheckError::UndefinedType(*name))
                }
            }
            _ => Err(TypeCheckError::UnsupportedTypeExpression),
        }
    }

    /// Check if a type name refers to a built-in type
    fn is_builtin_type(&self, type_name: InternedString) -> bool {
        type_name == self.builtins.String
            || type_name == self.builtins.UInt
            || type_name == self.builtins.Int
            || type_name == self.builtins.Bool
            || type_name == self.builtins.Double
    }

    /// Check a value expression and return its elaborated form
    fn check_value_expression<Loc>(
        &mut self,
        expr: &InternedExpression<Loc>,
    ) -> Result<elaborated::ValueExpression<InternedString>, TypeCheckError> {
        match &expr.node {
            ExpressionNode::Variable { name } => {
                let ty = self.context.get_type(*name).clone();
                Ok(elaborated::ValueExpression::Variable { name: *name, ty })
            }
            ExpressionNode::ConstructorCall { name, fields } => {
                let constructor = self
                    .constructors
                    .get(name)
                    .ok_or(TypeCheckError::UndefinedConstructor(*name))?
                    .clone();

                // Check field arguments
                let mut arguments = Vec::new();
                for field in fields {
                    let arg_expr = self.check_value_expression(&field.data)?;
                    arguments.push(arg_expr);
                }

                // Get implicit arguments from context
                let mut implicits = Vec::new();
                for (implicit_name, implicit_type) in &constructor.implicits {
                    implicits.push(elaborated::ValueExpression::Variable {
                        name: *implicit_name,
                        ty: implicit_type.clone(),
                    });
                }

                Ok(elaborated::ValueExpression::Constructor {
                    name: *name,
                    implicits: implicits.into(),
                    arguments: arguments.into(),
                    result_type: constructor.result_type.clone(),
                })
            }
            _ => Err(TypeCheckError::UnsupportedTypeExpression),
        }
    }

    /// Check a pattern and extract pattern variables
    fn check_pattern<Loc>(
        &mut self,
        pattern: &InternedPattern<Loc>,
        expected_type: &elaborated::TypeExpression<InternedString>,
    ) -> Result<Vec<(InternedString, elaborated::TypeExpression<InternedString>)>, TypeCheckError>
    {
        match &pattern.node {
            PatternNode::Variable { name } => {
                // Bind pattern variable to expected type
                self.context.bind_variable(*name, expected_type.clone())?;
                Ok(vec![(*name, expected_type.clone())])
            }
            PatternNode::ConstructorCall { name, fields } => {
                // Validate constructor pattern against expected type
                let constructor = self
                    .constructors
                    .get(name)
                    .ok_or(TypeCheckError::UndefinedConstructor(*name))?
                    .clone();

                let mut pattern_vars = Vec::new();
                for (i, field) in fields.iter().enumerate() {
                    if let Some((_, field_type)) = constructor.fields.get(i) {
                        let mut field_vars = self.check_pattern(&field.data, field_type)?;
                        pattern_vars.append(&mut field_vars);
                    }
                }

                Ok(pattern_vars)
            }
            _ => Ok(Vec::new()), // Literals, underscore, etc.
        }
    }

    /// Build the final elaborated module
    fn build_elaborated_module(&self) -> Result<Module<InternedString>, TypeCheckError> {
        // Convert types HashMap to sorted Vec by cloning
        let types: Vec<_> = self.types.clone().into_iter().collect();

        // Convert constructors HashMap to BTreeMap
        let constructors: std::collections::BTreeMap<_, _> =
            self.constructors.clone().into_iter().collect();

        Ok(Module {
            types,
            constructors,
        })
    }
}

// Additional trait implementations for Context to support the new methods
impl Context {
    /// Bind a variable to a type in the current scope layer
    pub fn bind_variable(
        &mut self,
        name: InternedString,
        ty: elaborated::TypeExpression<InternedString>,
    ) -> Result<(), TypeCheckError> {
        self.variables.insert(name, ty);
        Ok(())
    }

    /// Bind an alias to a value expression in the current scope layer
    pub fn bind_alias(
        &mut self,
        name: InternedString,
        expr: elaborated::ValueExpression<InternedString>,
    ) -> Result<(), TypeCheckError> {
        self.aliases.insert(name, expr);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_checker_creation() {
        let interner = StringInterner::new();
        let checker = TypeChecker::new(interner);
        assert!(checker.types.is_empty());
        assert!(checker.constructors.is_empty());
    }
}
