use dbuf_core::ast::elaborated::*;

pub type Str = String;

pub type ElaboratedAst = Module<Str>;

pub trait ElaboratedHelper {
    fn get_constructor_type(&self, constructor_name: &str) -> Option<&str>;
    fn get_type(&self, name: &str) -> Option<&Type<Str>>;
    fn get_any_constructor(&self, type_name: &str) -> Option<&Str>;
    fn has_type_or_constructor(&self, name: &str) -> bool;
    fn type_dependency_valid_rename(&self, type_name: &str, dependency: &str) -> bool;
    fn constructor_field_valid_rename(&self, constructor_name: &str, field: &str) -> bool;
}

fn constructor_has_field(ast: &ElaboratedAst, ctr: &str, field: &str) -> bool {
    if let Some(ctr) = ast.constructors.get(ctr) {
        if ctr.implicits.iter().any(|i| i.0 == field) {
            return true;
        }
        if ctr.fields.iter().any(|f| f.0 == field) {
            return true;
        }
        return false;
    }
    false
}

impl ElaboratedHelper for ElaboratedAst {
    fn get_constructor_type(&self, constructor_name: &str) -> Option<&str> {
        if let Some(ctr) = self.constructors.get(constructor_name) {
            if let Expression::Type {
                name,
                dependencies: _,
            } = &ctr.result_type
            {
                return Some(name);
            }
        }
        None
    }

    fn get_type(&self, name: &str) -> Option<&Type<Str>> {
        self.types
            .iter()
            .find(|(type_name, _)| type_name == name)
            .map(|(_, type_definition)| type_definition)
    }

    fn get_any_constructor(&self, type_name: &str) -> Option<&Str> {
        let t = self.get_type(type_name);
        if let Some(t) = t {
            match &t.constructor_names {
                ConstructorNames::OfMessage(ctr) => {
                    return Some(ctr);
                }
                ConstructorNames::OfEnum(ctrs) => {
                    if let Some(f) = ctrs.first() {
                        return Some(f);
                    } else {
                        return None;
                    }
                }
            }
        }
        None
    }

    fn has_type_or_constructor(&self, name: &str) -> bool {
        if self.types.iter().any(|t| t.0 == name) {
            return true;
        }
        if self.types.iter().any(|t| t.0 == name) {
            return true;
        }
        if self.constructors.keys().any(|ctr| name == ctr) {
            return true;
        }
        false
    }

    fn type_dependency_valid_rename(&self, type_name: &str, dependency: &str) -> bool {
        if let Some(t) = self.get_type(type_name) {
            if t.dependencies.iter().any(|d| d.0 == dependency) {
                return false;
            }
            match &t.constructor_names {
                ConstructorNames::OfMessage(ctr) => {
                    return !constructor_has_field(self, ctr, dependency)
                }
                ConstructorNames::OfEnum(ctrs) => {
                    return !ctrs
                        .iter()
                        .any(|ctr| constructor_has_field(self, ctr, dependency))
                }
            }
        }
        false
    }

    fn constructor_field_valid_rename(&self, constructor_name: &str, field: &str) -> bool {
        if let Some(type_name) = self.get_constructor_type(constructor_name) {
            return self.type_dependency_valid_rename(type_name, field);
        }
        false
    }
}
