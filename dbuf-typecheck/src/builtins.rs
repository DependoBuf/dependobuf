use crate::interning::{InternedString, StringInterner};

#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy)]
pub struct BuiltinTypes {
    pub String: InternedString,
    pub UInt: InternedString,
    pub Int: InternedString,
    pub Bool: InternedString,
    pub Double: InternedString,
}

impl BuiltinTypes {
    pub fn from_interner(interner: &mut StringInterner<String>) -> Self {
        Self {
            String: interner.transform("String"),
            UInt: interner.transform("UInt"),
            Int: interner.transform("Int"),
            Bool: interner.transform("Bool"),
            Double: interner.transform("Double"),
        }
    }
}
