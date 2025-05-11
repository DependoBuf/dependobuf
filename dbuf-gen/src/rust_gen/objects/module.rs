use std::collections::HashMap;

use crate::{
    format::{BoxDoc, DocAllocator},
    generate::{
        GlobalContext,
        lookup::{Cursor, LookupResult},
    },
};

use super::{GeneratedObject, GeneratedRustObject, Kind, Object, ObjectId, RustObject};

#[derive(Clone)]
pub struct Module<'id> {
    id: ObjectId<'id>,
    name: String,
}

#[derive(Debug, Clone)]
pub struct GeneratedModule {
    name: String,
    tag: u64,
}

fn format(name: String) -> String {
    // TODO: add r# or mandatory tag if ident is keyword
    name
}

fn tag_format(name: String, tag: u64) -> String {
    if tag > 0 {
        // here naming conventions must be applied
        format!("{}_{}", format(name), tag)
    } else {
        format(name)
    }
}

impl<'id> Module<'id> {
    pub fn from_object(id: ObjectId<'id>, name: String) -> Self {
        Self { id, name }
    }

    pub fn from_name(name: String) -> Self {
        Self {
            id: ObjectId::from_name(name.clone()),
            name,
        }
    }
}

impl<'id> Object<'id> for Module<'id> {
    type Generated = GeneratedModule;

    fn object_id(&self) -> ObjectId<'id> {
        self.id.clone()
    }

    fn kind() -> Kind {
        Kind::Module
    }

    fn backwards_lookup_limit(generated: &GeneratedRustObject) -> bool {
        match generated.kind() {
            Kind::Scope => false,
            Kind::Module => true,
            Kind::Type => false,
            Kind::Function => false,
            Kind::Variable => false,
        }
    }

    fn lookup_tag<'c, C>(cursor: C, object: &RustObject) -> Option<u64>
    where
        C: Cursor<(&'c HashMap<RustObject, u64>, &'c GeneratedRustObject), ObjectId<'id>>,
    {
        cursor
            .lookup(|_, (tags, generated)| match tags.get(object) {
                Some(tag) => LookupResult::Stop(Some(tag.clone())),
                None => {
                    if Self::backwards_lookup_limit(generated) {
                        LookupResult::Stop(None)
                    } else {
                        LookupResult::GoBack
                    }
                }
            })
            .flatten()
    }

    fn lookup_visible<'cursor, C>(cursor: C, id: ObjectId<'id>) -> Option<C>
    where
        C: Cursor<&'cursor GeneratedRustObject, ObjectId<'id>> + Clone,
    {
        cursor
            .lookup(|cursor, generated| {
                if cursor.key() == Some(&id) {
                    LookupResult::Stop(Some(cursor.clone()))
                } else {
                    match cursor.clone().next(&id) {
                        Some(cursor) => LookupResult::Stop(Some(cursor)),
                        None => {
                            if Self::backwards_lookup_limit(generated) {
                                LookupResult::Stop(None)
                            } else {
                                LookupResult::GoBack
                            }
                        }
                    }
                }
            })
            .flatten()
    }

    fn rust_object(&self) -> RustObject {
        RustObject::Module {
            name: format(self.name.clone()),
        }
    }

    fn generate_tagged(self, tag: u64) -> Self::Generated {
        GeneratedModule {
            name: self.name,
            tag,
        }
    }
}

impl GeneratedModule {
    pub fn to_doc<'a>(&self, ctx: GlobalContext<'a>) -> BoxDoc<'a> {
        ctx.alloc
            .text(tag_format(self.name.clone(), self.tag))
            .into_doc()
    }
}

impl From<GeneratedModule> for GeneratedRustObject {
    fn from(generated: GeneratedModule) -> Self {
        GeneratedRustObject {
            object: RustObject::Module {
                name: generated.name,
            },
            tag: generated.tag,
        }
    }
}

impl TryFrom<GeneratedRustObject> for GeneratedModule {
    type Error = ();

    fn try_from(generated: GeneratedRustObject) -> Result<Self, Self::Error> {
        if let RustObject::Module { name } = generated.object {
            Ok(GeneratedModule {
                name,
                tag: generated.tag,
            })
        } else {
            Err(())
        }
    }
}

impl TryFrom<&GeneratedRustObject> for GeneratedModule {
    type Error = ();

    fn try_from(value: &GeneratedRustObject) -> Result<Self, Self::Error> {
        Self::try_from(value.clone())
    }
}

impl<'me> GeneratedObject<'me> for GeneratedModule {}
