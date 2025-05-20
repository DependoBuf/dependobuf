use std::collections::HashMap;

use dbuf_core::ast::parsed::{ExpressionNode, TypeDefinition};
use thiserror::Error;

use crate::interning::{
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
}

type InternedSimpleTyperError = SimpleTyperError<InternedString>;
type Fields = HashMap<InternedString, InternedString>;

#[derive(Debug, Clone)]
pub enum SimpleType {
    Enum,
    Message { fields: Fields },
}

impl SimpleTyper {
    pub fn from_module<Loc>(module: &InternedModule<Loc>) -> Self {
        let mut typer: SimpleTyper = Default::default();
        for definition in module {
            typer.add_type(definition);
        }
        typer
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

    fn get_fields_from_constructor<Loc>(constructor: &InternedConstructor<Loc>) -> Fields {
        constructor
            .into_iter()
            .filter_map(|field| {
                let field_name = field.name.clone();
                let ExpressionNode::FunCall { fun: type_name, .. } = &field.node else {
                    return None;
                };
                Some((field_name, type_name.clone()))
            })
            .collect()
    }

    pub fn add_type<Loc>(&mut self, definition: &InternedTypeDeclaration<Loc>) {
        match &definition.body {
            TypeDefinition::Message(fields) => {
                let fields: Fields = Self::get_fields_from_constructor(fields);
                self.constructors.insert(definition.name, fields.clone());
                self.types
                    .insert(definition.name, SimpleType::Message { fields });
            }
            TypeDefinition::Enum(branches) => {
                self.types.insert(definition.name, SimpleType::Enum);
                for branch in branches {
                    for constructor in &branch.constructors {
                        self.constructors.insert(
                            constructor.name,
                            Self::get_fields_from_constructor(constructor),
                        );
                    }
                }
            }
        };
    }
}
