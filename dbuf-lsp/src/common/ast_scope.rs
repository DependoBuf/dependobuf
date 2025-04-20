use dbuf_core::ast::elaborated::*;

use crate::common::ast_access::ElaboratedHelper;

use super::ast_access::ElaboratedAst;

struct Cache<'a> {
    type_name: &'a str,
    constructor_name: &'a str,
}

impl<'a> From<&mut AstScope<'a>> for Cache<'a> {
    fn from(value: &mut AstScope<'a>) -> Cache<'a> {
        Cache {
            type_name: value.type_name,
            constructor_name: value.constructor_name,
        }
    }
}

pub struct AstScope<'a> {
    elaborated: &'a ElaboratedAst,

    type_name: &'a str,
    constructor_name: &'a str,

    cache: Option<Cache<'a>>,
}

impl<'a> AstScope<'a> {
    pub fn new(elaborated: &ElaboratedAst) -> AstScope {
        AstScope {
            elaborated,
            type_name: "",
            constructor_name: "",
            cache: None,
        }
    }

    pub fn get_type(&self) -> &'a str {
        assert!(!self.type_name.is_empty());
        self.type_name
    }

    pub fn get_option_type(&self) -> &'a str {
        self.type_name
    }

    pub fn get_constructor(&self) -> &'a str {
        assert!(!self.constructor_name.is_empty());
        self.constructor_name
    }

    pub fn get_option_constructor(&self) -> &'a str {
        self.constructor_name
    }

    pub fn enter_into_type(&mut self, type_name: &'a str) {
        assert!(
            self.elaborated.has_type(type_name),
            "no type in elaborated ast"
        );

        self.type_name = type_name;
        self.constructor_name = "";
    }

    pub fn enter_into_constructor(&mut self, constructor_name: &'a str) {
        assert!(!self.type_name.is_empty());
        assert!(
            self.elaborated
                .is_type_constructor(self.type_name, constructor_name),
            "type hasn't got that constructor"
        );

        self.constructor_name = constructor_name;
    }

    pub fn exit_type(&mut self) {
        self.type_name = "";
    }

    pub fn exit_constructor(&mut self) {
        self.constructor_name = "";
    }

    pub fn enter_into_message(&mut self, message: &'a str) {
        self.enter_into_type(message);
        self.enter_into_constructor(message);
    }

    fn switch_to_type(&mut self, type_name: &'a str) {
        if self.elaborated.is_message(type_name) {
            self.enter_into_message(type_name);
        } else {
            self.enter_into_type(type_name);
        }
    }

    fn try_switch_to(
        &mut self,
        variable: &str,
        variants: &'a [(String, Expression<String>)],
    ) -> Option<&'a str> {
        if let Some((
            _,
            TypeExpression::Type {
                name,
                dependencies: _,
            },
        )) = variants.iter().rev().find(|v| v.0 == variable)
        {
            Some(name)
        } else {
            None
        }
    }

    pub fn apply_variable(&mut self, variable: &str) {
        assert!(!self.type_name.is_empty(), "unknow type to apply variable");

        let t = self.elaborated.get_type(self.type_name).unwrap();

        let mut switch = self.try_switch_to(variable, &t.dependencies).unwrap_or("");

        if !self.constructor_name.is_empty() {
            let c = self
                .elaborated
                .get_constructor(self.constructor_name)
                .unwrap();
            switch = self.try_switch_to(variable, &c.implicits).unwrap_or(switch);
            switch = self.try_switch_to(variable, &c.fields).unwrap_or(switch);
        }

        assert!(!switch.is_empty(), "unknow variable to switch");

        self.switch_to_type(switch);
    }

    pub fn save_state(&mut self) {
        assert!(self.cache.is_none(), "no saved state");

        self.cache = Some(self.into());
    }

    pub fn load_state(&mut self) {
        assert!(self.cache.is_some(), "has saved state");

        let cache = self.cache.take().unwrap();
        self.type_name = cache.type_name;
        self.constructor_name = cache.constructor_name;
    }
}
