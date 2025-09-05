use std::collections::HashMap;

use crate::ast::elaborated::{Constructor, Type};

use super::{
    builtins::BuiltinTypes,
    context::Context,
    interning::{InternedString, StringInterner},
};

pub struct AdvancedTyper {
    _context: Box<Context>,
    pub interner: StringInterner<String>,
    _builtins: BuiltinTypes,
    _constructors: HashMap<InternedString, Constructor<InternedString>>,
    _types: HashMap<InternedString, Type<InternedString>>,
}

impl AdvancedTyper {}
