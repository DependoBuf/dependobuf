use std::collections::HashMap;

use crate::ast::elaborated::{TypeExpression, ValueExpression};

use super::interning::InternedString;

#[derive(Default, Debug, Clone)]
pub struct Context {
    parent: Option<Box<Context>>,
    pub variables: HashMap<InternedString, TypeExpression<InternedString>>,
    pub aliases: HashMap<InternedString, ValueExpression<InternedString>>,
}

impl Context {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_layer(self: Box<Self>) -> Box<Self> {
        Box::new(Context {
            parent: Some(self),
            ..Default::default()
        })
    }

    pub fn pop_layer(self: Box<Self>) -> Box<Self> {
        self.parent.unwrap()
    }

    pub fn find_layer(self: &mut Box<Self>, name: InternedString) -> &mut Box<Self> {
        if self.variables.contains_key(&name) {
            return self;
        }
        self.parent
            .as_mut()
            .expect("variable not found")
            .find_layer(name)
    }

    pub fn find_alias(&self, name: InternedString) -> Option<&ValueExpression<InternedString>> {
        self.aliases
            .get(&name)
            .or_else(|| self.parent.as_ref()?.find_alias(name))
    }

    pub fn get_type(&self, name: InternedString) -> &TypeExpression<InternedString> {
        self.variables.get(&name).unwrap_or_else(|| {
            self.parent
                .as_ref()
                .expect("variable not found")
                .get_type(name)
        })
    }
}
