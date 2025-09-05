#[cfg(test)]
mod integration_tests {
    use crate::ast::parsed::{
        definition::Definition, location::Location, EnumBranch, Expression, ExpressionNode,
        TypeDeclaration, TypeDefinition,
    };
    use crate::typecheck::{
        BuiltinTypes, ModuleInterner, StrategyBuilder, TypeCheckError, TypeChecker,
    };

    type TestLocation = Location<usize>;
    type TestExpression = Expression<TestLocation, String>;
    type TestDefinition<T> = Definition<TestLocation, String, T>;

    /// Test type checking a simple message type
    #[test]
    fn test_simple_message_type_checking() {
        // Create a simple message type:
        // message Person {
        //     name: String
        //     age: UInt
        // }

        // Create the parsed AST manually for testing
        let name_field = TestDefinition {
            loc: TestLocation::default(),
            name: "name".to_string(),
            data: TestExpression {
                loc: TestLocation::default(),
                node: ExpressionNode::FunCall {
                    fun: "String".to_string(),
                    args: std::sync::Arc::new([]),
                },
            },
        };

        let age_field = TestDefinition {
            loc: TestLocation::default(),
            name: "age".to_string(),
            data: TestExpression {
                loc: TestLocation::default(),
                node: ExpressionNode::FunCall {
                    fun: "UInt".to_string(),
                    args: std::sync::Arc::new([]),
                },
            },
        };

        let person_type = TestDefinition {
            loc: TestLocation::default(),
            name: "Person".to_string(),
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Message(vec![name_field, age_field]),
            },
        };

        let module = vec![person_type];

        // Transform to interned module
        let mut module_interner = ModuleInterner::new();

        // Create builtin types BEFORE transforming the module so they get consistent IDs
        let builtins = BuiltinTypes::from_interner(&mut module_interner.interner);

        let interned_module = module_interner.transform_module(module);

        // Create type checker with the interner and builtin types
        let mut type_checker = TypeChecker::with_builtins(module_interner.interner, builtins);

        // Check the module
        let result = type_checker.check_module(&interned_module);

        // Should succeed
        assert!(result.is_ok(), "Type checking failed: {:?}", result.err());

        let elaborated_module = result.unwrap();

        // Verify the elaborated module has the expected structure
        assert_eq!(elaborated_module.types.len(), 1);
        assert_eq!(elaborated_module.constructors.len(), 1);
    }

    /// Test type checking an enum with multiple constructors
    #[test]
    fn test_enum_type_checking() {
        // Create an enum type:
        // enum Color {
        //     | Red()
        //     | Green()
        //     | Blue()
        // }

        let red_constructor = TestDefinition {
            loc: TestLocation::default(),
            name: "Red".to_string(),
            data: vec![], // No fields - this is ConstructorBody
        };

        let green_constructor = TestDefinition {
            loc: TestLocation::default(),
            name: "Green".to_string(),
            data: vec![], // No fields
        };

        let blue_constructor = TestDefinition {
            loc: TestLocation::default(),
            name: "Blue".to_string(),
            data: vec![], // No fields
        };

        let color_branch = EnumBranch {
            patterns: vec![],
            constructors: vec![red_constructor, green_constructor, blue_constructor],
        };

        let color_type = TestDefinition {
            loc: TestLocation::default(),
            name: "Color".to_string(),
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Enum(vec![color_branch]),
            },
        };

        let module = vec![color_type];

        // Transform to interned module
        let mut module_interner = ModuleInterner::new();

        // Create builtin types BEFORE transforming the module so they get consistent IDs
        let builtins = BuiltinTypes::from_interner(&mut module_interner.interner);

        let interned_module = module_interner.transform_module(module);

        // Create type checker
        let mut type_checker = TypeChecker::with_builtins(module_interner.interner, builtins);

        // Check the module
        let result = type_checker.check_module(&interned_module);

        // Should succeed
        assert!(
            result.is_ok(),
            "Enum type checking failed: {:?}",
            result.err()
        );

        let elaborated_module = result.unwrap();

        // Verify the elaborated module has the expected structure
        assert_eq!(elaborated_module.types.len(), 1);
        assert_eq!(elaborated_module.constructors.len(), 3); // Red, Green, Blue
    }

    /// Test type checking with dependencies
    #[test]
    fn test_dependent_type_checking() {
        // Create types with dependencies:
        // message Address {
        //     street: String
        //     city: String
        // }
        // message Person {
        //     name: String
        //     address: Address
        // }

        // Address type
        let street_field = TestDefinition {
            loc: TestLocation::default(),
            name: "street".to_string(),
            data: TestExpression {
                loc: TestLocation::default(),
                node: ExpressionNode::FunCall {
                    fun: "String".to_string(),
                    args: std::sync::Arc::new([]),
                },
            },
        };

        let city_field = TestDefinition {
            loc: TestLocation::default(),
            name: "city".to_string(),
            data: TestExpression {
                loc: TestLocation::default(),
                node: ExpressionNode::FunCall {
                    fun: "String".to_string(),
                    args: std::sync::Arc::new([]),
                },
            },
        };

        let address_type = TestDefinition {
            loc: TestLocation::default(),
            name: "Address".to_string(),
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Message(vec![street_field, city_field]),
            },
        };

        // Person type that depends on Address
        let name_field = TestDefinition {
            loc: TestLocation::default(),
            name: "name".to_string(),
            data: TestExpression {
                loc: TestLocation::default(),
                node: ExpressionNode::FunCall {
                    fun: "String".to_string(),
                    args: std::sync::Arc::new([]),
                },
            },
        };

        let address_field = TestDefinition {
            loc: TestLocation::default(),
            name: "address".to_string(),
            data: TestExpression {
                loc: TestLocation::default(),
                node: ExpressionNode::FunCall {
                    fun: "Address".to_string(),
                    args: std::sync::Arc::new([]),
                },
            },
        };

        let person_type = TestDefinition {
            loc: TestLocation::default(),
            name: "Person".to_string(),
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Message(vec![name_field, address_field]),
            },
        };

        let module = vec![address_type, person_type];

        // Transform to interned module
        let mut module_interner = ModuleInterner::new();

        // Create builtin types BEFORE transforming the module so they get consistent IDs
        let builtins = BuiltinTypes::from_interner(&mut module_interner.interner);

        let interned_module = module_interner.transform_module(module);

        // Create type checker
        let mut type_checker = TypeChecker::with_builtins(module_interner.interner, builtins);

        // Check the module
        let result = type_checker.check_module(&interned_module);

        // Should succeed
        assert!(
            result.is_ok(),
            "Dependent type checking failed: {:?}",
            result.err()
        );

        let elaborated_module = result.unwrap();

        // Verify the elaborated module has the expected structure
        assert_eq!(elaborated_module.types.len(), 2); // Address and Person
        assert_eq!(elaborated_module.constructors.len(), 2); // Address and Person constructors
    }

    /// Test error case - undefined type reference
    #[test]
    fn test_undefined_type_error() {
        // Person type referencing undefined "UnknownType"
        let unknown_field = TestDefinition {
            loc: TestLocation::default(),
            name: "unknown".to_string(),
            data: TestExpression {
                loc: TestLocation::default(),
                node: ExpressionNode::FunCall {
                    fun: "UnknownType".to_string(),
                    args: std::sync::Arc::new([]),
                },
            },
        };

        let person_type = TestDefinition {
            loc: TestLocation::default(),
            name: "Person".to_string(),
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Message(vec![unknown_field]),
            },
        };

        let module = vec![person_type];

        // Transform to interned module
        let mut module_interner = ModuleInterner::new();

        // Create builtin types BEFORE transforming the module so they get consistent IDs
        let builtins = BuiltinTypes::from_interner(&mut module_interner.interner);

        let interned_module = module_interner.transform_module(module);

        // Create type checker
        let mut type_checker = TypeChecker::with_builtins(module_interner.interner, builtins);

        // Check the module - should fail
        let result = type_checker.check_module(&interned_module);

        // Should fail with undefined type error
        assert!(
            result.is_err(),
            "Expected type checking to fail for undefined type"
        );

        // Check that it's the right kind of error
        if let Err(TypeCheckError::UndefinedType(_)) = result {
            // Expected error type
            println!("Got expected UndefinedType error");
        } else {
            panic!("Expected UndefinedType error, got: {:?}", result);
        }
    }

    /// Test the complete type checking workflow with the strategy builder
    #[test]
    fn test_strategy_builder_integration() {
        // Simple message for testing strategy building
        let name_field = TestDefinition {
            loc: TestLocation::default(),
            name: "name".to_string(),
            data: TestExpression {
                loc: TestLocation::default(),
                node: ExpressionNode::FunCall {
                    fun: "String".to_string(),
                    args: std::sync::Arc::new([]),
                },
            },
        };

        let person_type = TestDefinition {
            loc: TestLocation::default(),
            name: "Person".to_string(),
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Message(vec![name_field]),
            },
        };

        let module = vec![person_type];

        // Transform to interned module
        let mut module_interner = ModuleInterner::new();

        // Create builtin types BEFORE transforming the module so they get consistent IDs
        let builtins = BuiltinTypes::from_interner(&mut module_interner.interner);

        let interned_module = module_interner.transform_module(module);

        // Test strategy building separately
        let strategy_result = StrategyBuilder::build_strategy(&interned_module);
        assert!(
            strategy_result.is_ok(),
            "Strategy building failed: {:?}",
            strategy_result.err()
        );

        let tasks = strategy_result.unwrap();

        // Should have at least signature and constructor tasks
        assert!(!tasks.is_empty(), "Strategy should produce tasks");

        println!("Generated tasks: {:?}", tasks);

        // Now test full type checking
        let mut type_checker = TypeChecker::with_builtins(module_interner.interner, builtins);
        let result = type_checker.check_module(&interned_module);

        assert!(
            result.is_ok(),
            "Full type checking failed: {:?}",
            result.err()
        );
    }

    /// Test unsupported type expression error
    #[test]
    fn test_unsupported_type_expression_error() {
        // Test case for unsupported expression types
        let unsupported_field = TestDefinition {
            loc: TestLocation::default(),
            name: "field".to_string(),
            data: TestExpression {
                loc: TestLocation::default(),
                node: ExpressionNode::Variable {
                    name: "SomeType".to_string(), // Variable reference (not FunCall) is unsupported in type context
                },
            },
        };

        let test_type = TestDefinition {
            loc: TestLocation::default(),
            name: "TestType".to_string(),
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Message(vec![unsupported_field]),
            },
        };

        let module = vec![test_type];

        // Transform to interned module
        let mut module_interner = ModuleInterner::new();

        // Create builtin types BEFORE transforming the module so they get consistent IDs
        let builtins = BuiltinTypes::from_interner(&mut module_interner.interner);

        let interned_module = module_interner.transform_module(module);

        // Create type checker
        let mut type_checker = TypeChecker::with_builtins(module_interner.interner, builtins);

        // Check the module - should fail
        let result = type_checker.check_module(&interned_module);

        // Should fail with unsupported type expression error
        assert!(
            result.is_err(),
            "Expected type checking to fail for unsupported type expression"
        );

        // Check that it's the right kind of error
        match result {
            Err(TypeCheckError::UnsupportedTypeExpression) => {
                println!("Got expected UnsupportedTypeExpression error");
            }
            Err(other_err) => {
                // May also fail with SimpleTyperError::Unsupported which is also acceptable
                println!("Got error (acceptable): {:?}", other_err);
            }
            Ok(_) => panic!("Expected error but got success"),
        }
    }
}
