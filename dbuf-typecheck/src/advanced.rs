use std::collections::HashMap;

use dbuf_core::ast::elaborated::{Constructor, Type};

use crate::{
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
