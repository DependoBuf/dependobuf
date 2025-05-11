use std::collections::HashMap;

use crate::generate::lookup::Cursor;

use super::{GeneratedObject, GeneratedRustObject, Kind, Object, ObjectId, RustObject};

#[derive(Clone)]
pub struct Scope<'id> {
    id: ObjectId<'id>,
}

#[derive(Debug, Clone)]
pub struct GeneratedScope {
    tag: u64,
}

impl<'id> Scope<'id> {
    pub fn new(id: ObjectId<'id>) -> Self {
        Self { id }
    }
}

impl<'id> Object<'id> for Scope<'id> {
    type Generated = GeneratedScope;

    fn object_id(&self) -> ObjectId<'id> {
        self.id.clone()
    }

    fn kind() -> Kind {
        Kind::Scope
    }

    fn backwards_lookup_limit(_: &GeneratedRustObject) -> bool {
        true
    }

    fn lookup_tag<'c, C>(cursor: C, object: &RustObject) -> Option<u64>
    where
        C: Cursor<(&'c HashMap<RustObject, u64>, &'c GeneratedRustObject), ObjectId<'id>>,
    {
        cursor.value().0.get(object).cloned()
    }

    fn lookup_visible<'cursor, C>(cursor: C, id: ObjectId<'id>) -> Option<C>
    where
        C: Cursor<&'cursor GeneratedRustObject, ObjectId<'id>> + Clone,
    {
        cursor.next(&id)
    }

    fn rust_object(&self) -> RustObject {
        RustObject::Scope
    }

    fn generate_tagged(self, tag: u64) -> Self::Generated {
        GeneratedScope { tag }
    }
}

impl From<GeneratedScope> for GeneratedRustObject {
    fn from(generated: GeneratedScope) -> Self {
        GeneratedRustObject {
            object: RustObject::Scope,
            tag: generated.tag,
        }
    }
}

impl TryFrom<GeneratedRustObject> for GeneratedScope {
    type Error = ();

    fn try_from(generated: GeneratedRustObject) -> Result<Self, Self::Error> {
        if let RustObject::Scope = generated.object {
            Ok(GeneratedScope { tag: generated.tag })
        } else {
            Err(())
        }
    }
}

impl TryFrom<&GeneratedRustObject> for GeneratedScope {
    type Error = ();

    fn try_from(value: &GeneratedRustObject) -> Result<Self, Self::Error> {
        Self::try_from(value.clone())
    }
}

impl<'me> GeneratedObject<'me> for GeneratedScope {}
