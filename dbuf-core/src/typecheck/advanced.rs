use std::collections::HashMap;

use crate::ast::elaborated::{Constructor, Type};

use super::{
    builtins::BuiltinTypes,
    context::Context,
    interning::{InternedString, StringInterner},
};

pub struct AdvancedTyper {
    context: Box<Context>,
    pub interner: StringInterner<String>,
    builtins: BuiltinTypes,
    constructors: HashMap<InternedString, Constructor<InternedString>>,
    types: HashMap<InternedString, Type<InternedString>>,
}

impl AdvancedTyper {}
