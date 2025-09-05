use std::collections::HashMap;

use crate::ast::parsed::{ExpressionNode, TypeDefinition};
use thiserror::Error;

use super::interning::{
    InternedConstructor, InternedModule, InternedString, InternedTypeDeclaration,
};

#[derive(Default, Debug)]
pub struct SimpleTyper {
    types: HashMap<InternedString, SimpleType>,
    constructors: HashMap<InternedString, Fields>,
}

#[derive(Error, Debug)]
pub enum SimpleTyperError<Str> {
    #[error("unknown type `{0}`")]
    UnknownType(Str),
    #[error("unknown constructor `{0}`")]
    UnknownConstructor(Str),
    #[error("type `{0}` is not a message")]
    NotAMessage(Str),
    #[error("type `{name}` doesn't have field `{field}`")]
    UnknownFieldMessage { name: Str, field: Str },
    #[error("constructor `{name}` doesn't have field `{field}`")]
    UnknownFieldConstructor { name: Str, field: Str },
    #[error("field `{field}` in constructor `{name}` has unsupported type")]
    Unsupported { name: Str, field: Str },
}

type InternedSimpleTyperError = SimpleTyperError<InternedString>;
type Fields = HashMap<InternedString, InternedString>;

#[derive(Debug, Clone)]
pub enum SimpleType {
    Enum,
    Message { fields: Fields },
}

impl SimpleTyper {
    pub fn from_module<Loc>(
        module: &InternedModule<Loc>,
    ) -> Result<Self, InternedSimpleTyperError> {
        let mut typer: SimpleTyper = Default::default();
        for definition in module {
            typer.add_type(definition)?;
        }
        Ok(typer)
    }

    pub fn get_field(
        &self,
        name: InternedString,
        field: InternedString,
    ) -> Result<InternedString, InternedSimpleTyperError> {
        let r#type = self
            .types
            .get(&name)
            .ok_or_else(|| SimpleTyperError::UnknownType(name.clone()))?;
        let SimpleType::Message { fields } = r#type else {
            return Err(SimpleTyperError::NotAMessage(name.clone()));
        };
        fields
            .get(&field)
            .ok_or_else(|| SimpleTyperError::UnknownFieldMessage {
                name: name.clone(),
                field: field.clone(),
            })
            .cloned()
    }

    pub fn get_field_from_constructor(
        &self,
        name: InternedString,
        field: InternedString,
    ) -> Result<InternedString, InternedSimpleTyperError> {
        let fields = self
            .constructors
            .get(&name)
            .ok_or_else(|| SimpleTyperError::UnknownType(name.clone()))?;
        fields
            .get(&field)
            .ok_or_else(|| SimpleTyperError::UnknownFieldConstructor {
                name: name.clone(),
                field: field.clone(),
            })
            .cloned()
    }

    fn get_fields_from_constructor<Loc>(
        constructor_name: &InternedString,
        constructor: &InternedConstructor<Loc>,
    ) -> Result<Fields, InternedSimpleTyperError> {
        let mut fields = Fields::new();
        for field in constructor {
            let field_name = field.name.clone();
            let ExpressionNode::FunCall { fun: type_name, .. } = &field.node else {
                return Err(SimpleTyperError::Unsupported {
                    name: constructor_name.clone(),
                    field: field_name.clone(),
                });
            };
            fields.insert(field_name, type_name.clone());
        }
        Ok(fields)
    }

    pub fn add_type<Loc>(
        &mut self,
        definition: &InternedTypeDeclaration<Loc>,
    ) -> Result<(), InternedSimpleTyperError> {
        match &definition.body {
            TypeDefinition::Message(fields) => {
                let fields: Fields = Self::get_fields_from_constructor(&definition.name, fields)?;
                self.constructors.insert(definition.name, fields.clone());
                self.types
                    .insert(definition.name, SimpleType::Message { fields });
            }
            TypeDefinition::Enum(branches) => {
                self.types.insert(definition.name, SimpleType::Enum);
                for branch in branches {
                    for constructor in &branch.constructors {
                        let constructor_fields =
                            Self::get_fields_from_constructor(&constructor.name, constructor)?;
                        self.constructors
                            .insert(constructor.name, constructor_fields);
                    }
                }
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parsed::location::Location;
    use crate::ast::parsed::{
        definition::Definition, ExpressionNode, TypeDeclaration, TypeDefinition,
    };
    use std::sync::Arc;

    type TestLocation = Location<usize>;

    #[test]
    fn test_unsupported_field_type() {
        let mut interner = crate::typecheck::interning::ModuleInterner::<String>::new();

        let field = Definition {
            loc: TestLocation::default(),
            name: "field1".to_string(),
            data: crate::ast::parsed::Expression {
                loc: TestLocation::default(),
                node: ExpressionNode::Variable {
                    name: "SomeType".to_string(),
                },
            },
        };

        let type_def = Definition {
            loc: TestLocation::default(),
            name: "TestType".to_string(),
            data: TypeDeclaration {
                dependencies: vec![], // no dependencies
                body: TypeDefinition::Message(vec![field]),
            },
        };

        let module = vec![type_def];
        let interned_module = interner.transform_module(module);

        let result = SimpleTyper::from_module(&interned_module);
        assert!(result.is_err());

        if let Err(SimpleTyperError::Unsupported { .. }) = result {
            // Expected error type
        } else {
            panic!("Expected Unsupported error, got: {:?}", result);
        }
    }

    #[test]
    fn test_supported_field_type() {
        let mut interner = crate::typecheck::interning::ModuleInterner::<String>::new();

        let field = Definition {
            loc: TestLocation::default(),
            name: "field1".to_string(),
            data: crate::ast::parsed::Expression {
                loc: TestLocation::default(),
                node: ExpressionNode::FunCall {
                    fun: "SomeType".to_string(),
                    args: Arc::new([]),
                },
            },
        };

        let type_def = Definition {
            loc: TestLocation::default(),
            name: "TestType".to_string(),
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Message(vec![field]),
            },
        };

        let module = vec![type_def];
        let interned_module = interner.transform_module(module);

        let result = SimpleTyper::from_module(&interned_module);
        assert!(result.is_ok());
    }
}
