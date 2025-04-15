use dbuf_core::ast::elaborated::*;

pub type Str = String;

pub type ElaboratedAst = Module<Str>;

pub trait ElaboratedHelper {
    fn get_type(&self, name: &str) -> Option<&Type<Str>>;
    fn get_any_constructor(&self, type_name: &str) -> Option<&Str>;
}

impl ElaboratedHelper for ElaboratedAst {
    fn get_type(&self, name: &str) -> Option<&Type<Str>> {
        for (type_name, type_definition) in self.types.iter() {
            if type_name != name {
                continue;
            }
            return Some(type_definition);
        }
        return None;
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
}
