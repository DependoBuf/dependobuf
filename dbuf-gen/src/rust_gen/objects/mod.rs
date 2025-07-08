mod function;
mod module;
mod scope;
mod ty;
mod variable;

use std::collections::HashMap;

use crate::{ast::NodeId, generate::lookup::Cursor};

pub use function::{Function, GeneratedFunction};
pub use module::{GeneratedModule, Module};
#[allow(unused_imports, reason = "scopes currently are not generated ?")]
pub use scope::{GeneratedScope, Scope};
pub use ty::{GeneratedType, Type};
#[allow(unused_imports, reason = "variable currently are not generated ?")]
pub use variable::{GeneratedVariable, Variable};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Kind {
    Scope,
    Module,
    Type,
    Function,
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RustObject {
    Scope,
    Module { name: String },
    Type { name: String },
    Function { name: String },
    Variable { name: String },
    // Macro?
}

#[derive(Debug, Clone)]
pub struct GeneratedRustObject {
    object: RustObject,
    tag: u64,
}

#[allow(dead_code, reason = "??? (some items are never used)")]
pub trait Object<'id>: Sized {
    type Generated: for<'me> GeneratedObject<'me>;

    fn object_id(&self) -> ObjectId<'id>;

    fn kind() -> Kind;

    fn backwards_lookup_limit(generated: &GeneratedRustObject) -> bool;

    fn lookup_tag<'c, C>(cursor: C, object: &RustObject) -> Option<u64>
    where
        C: Cursor<(&'c HashMap<RustObject, u64>, &'c GeneratedRustObject), ObjectId<'id>>;

    fn lookup_visible<'c, C>(cursor: C, id: ObjectId<'id>) -> Option<C>
    where
        C: Cursor<&'c GeneratedRustObject, ObjectId<'id>> + Clone;

    fn rust_object(&self) -> RustObject;

    fn generate(self) -> Self::Generated {
        self.generate_tagged(0)
    }

    fn generate_tagged(self, tag: u64) -> Self::Generated;
}

#[allow(dead_code, reason = "??? (trait never used)")]
pub trait GeneratedObject<'me>:
    Into<GeneratedRustObject>
    + TryFrom<GeneratedRustObject, Error = ()>
    + TryFrom<&'me GeneratedRustObject, Error = ()>
    + Clone
{
}

#[allow(dead_code, reason = "??? (Number never constructed)")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tag {
    None,
    Number(u64),
    String(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectId<'id>(pub NodeId<'id>, pub Tag);

#[allow(dead_code, reason = "??? (some methods are never used)")]
impl ObjectId<'_> {
    pub fn from_name(name: String) -> Self {
        ObjectId(NodeId::owned(name), Tag::None)
    }

    pub fn with_tag(mut self, tag: Tag) -> Self {
        self.1 = tag;
        self
    }
}

impl RustObject {
    pub fn kind(&self) -> Kind {
        match self {
            RustObject::Scope => Kind::Scope,
            RustObject::Module { name: _ } => Kind::Module,
            RustObject::Type { name: _ } => Kind::Type,
            RustObject::Function { name: _ } => Kind::Function,
            RustObject::Variable { name: _ } => Kind::Variable,
        }
    }
}

impl GeneratedRustObject {
    pub fn kind(&self) -> Kind {
        self.object.kind()
    }
}
