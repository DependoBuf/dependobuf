use std::{collections::HashMap, hash::Hash, marker::PhantomData, mem::ManuallyDrop};

pub use crate::generate::node;

use crate::{
    ast::NodeId,
    generate::{
        GlobalContext,
        lookup::{Cursor, LookupResult, NodeCursor},
        namespace::NamespaceTree,
    },
};

use super::objects::{self, GeneratedRustObject, Kind, Object, ObjectId, RustObject};

// TODO: need to introduce IntoCursor trait that is analogues to IntoIterator. Because for now I need to write really awful constructions such as:
// context::GeneratedCursor<'cursor, 'a, impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>

pub type MutContext<'a, 'parent, 'me> = (GlobalContext<'a>, &'me mut NamingContext<'a, 'parent>);

pub type Context<'a, 'me, C> = (GlobalContext<'a>, GeneratedCursor<'me, 'a, C>);

pub type Node<'id> = node::Node<ObjectId<'id>, Container>;

#[derive(Debug, Clone)]
pub struct Container {
    object: GeneratedRustObject,
    tags: HashMap<RustObject, u64>,
}

#[derive(Debug)]
pub struct NamingContext<'id, 'parent>(
    ManuallyDrop<NamespaceTree<'parent, ObjectId<'id>, Container>>,
);

#[derive(Debug, Clone)]
pub struct GeneratedCursor<'cursor, 'id, C>
where
    C: Cursor<&'cursor GeneratedRustObject, ObjectId<'id>>,
{
    cursor: C,
    _value: PhantomData<&'cursor GeneratedRustObject>,
    _key: PhantomData<&'cursor ObjectId<'id>>,
}

#[allow(dead_code, reason = "??? (object is never read)")]
#[derive(Clone)]
pub struct Name {
    object: RustObject,
    tag: u64,
}

#[allow(dead_code, reason = "??? (some methods are never used)")]
impl<'id> NamingContext<'id, '_> {
    fn wrap_namespace_tree<'a>(
        namespace_tree: NamespaceTree<'a, ObjectId<'id>, Container>,
    ) -> NamingContext<'id, 'a> {
        NamingContext(ManuallyDrop::new(namespace_tree))
    }

    fn insert_object_with_tag<'child, O: Object<'id>>(
        &'child mut self,
        object: O,
        tag: u64,
    ) -> (O::Generated, NamingContext<'id, 'child>) {
        self.0.get_mut().tags.insert(object.rust_object(), tag);

        let id = object.object_id();
        let generated = object.generate_tagged(tag);
        (
            generated.clone(),
            Self::wrap_namespace_tree(self.0.insert(
                id,
                Container {
                    object: generated.into(),
                    tags: HashMap::new(),
                },
            )),
        )
    }

    pub fn root() -> Self {
        Self::wrap_namespace_tree(NamespaceTree::root(Container {
            object: objects::Scope::new(ObjectId(
                NodeId::owned("root".to_owned()),
                objects::Tag::None,
            ))
            .generate_tagged(0)
            .into(),
            tags: HashMap::new(),
        }))
    }

    pub fn insert_tree<'cursor>(
        &'cursor mut self,
        id: &ObjectId<'id>,
        tree: Node<'id>,
    ) -> GeneratedCursor<
        'cursor,
        'id,
        impl NodeCursor<Container, ObjectId<'id>> + Cursor<&'cursor GeneratedRustObject, ObjectId<'id>>,
    > {
        self.0.insert_tree(id.clone(), tree);

        GeneratedCursor {
            cursor: self
                .cursor()
                .next(id)
                .expect("newly inserted tree couldn't be accessed"),
            _value: PhantomData,
            _key: PhantomData,
        }
    }

    pub fn remove_tree(&mut self, id: &ObjectId<'id>) -> Option<Node<'id>> {
        self.0.remove_tree(id)
    }

    pub fn name_object<O: Object<'id>>(&mut self, object: &O) -> Name {
        let tag = match O::lookup_tag(
            self.0.cursor().value_map(|node| (&node.tags, &node.object)),
            &object.rust_object(),
        ) {
            Some(tag) => tag + 1,
            None => 0,
        };
        self.0.get_mut().tags.insert(object.rust_object(), tag);
        Name {
            object: object.rust_object(),
            tag,
        }
    }

    pub fn insert_object<'child, O: Object<'id>>(
        &'child mut self,
        object: O,
        name: &Name,
    ) -> (O::Generated, NamingContext<'id, 'child>) {
        let id = object.object_id();
        let generated = object.generate_tagged(name.tag);
        (
            generated.clone(),
            Self::wrap_namespace_tree(self.0.insert(
                id,
                Container {
                    object: generated.into(),
                    tags: HashMap::new(),
                },
            )),
        )
    }

    pub fn insert_object_preserve_name<'child, O: Object<'id>>(
        &'child mut self,
        object: O,
    ) -> Option<(O::Generated, NamingContext<'id, 'child>)> {
        let rust_object = object.rust_object();
        match O::lookup_tag(
            self.0.cursor().value_map(|node| (&node.tags, &node.object)),
            &rust_object,
        ) {
            Some(_) => None,
            None => Some(self.insert_object(
                object,
                &Name {
                    object: rust_object,
                    tag: 0,
                },
            )),
        }
    }

    pub fn insert_object_auto_name<'child, O: Object<'id>>(
        &'child mut self,
        object: O,
    ) -> (O::Generated, NamingContext<'id, 'child>) {
        let name = self.name_object(&object);
        self.insert_object(object, &name)
    }

    pub fn cursor<'cursor>(
        &'cursor self,
    ) -> GeneratedCursor<
        'cursor,
        'id,
        impl NodeCursor<Container, ObjectId<'id>> + Cursor<&'cursor GeneratedRustObject, ObjectId<'id>>,
    > {
        GeneratedCursor {
            cursor: self.0.cursor().value_map(|node| &node.object),
            _value: PhantomData,
            _key: PhantomData,
        }
    }

    #[allow(
        clippy::type_complexity,
        reason = "type TypeName<'lifetime> = ... is unstable"
    )]
    pub fn get_generated<'cursor, O: Object<'id>>(
        &'cursor self,
        id: ObjectId<'id>,
    ) -> Option<(
        O::Generated,
        GeneratedCursor<
            'cursor,
            'id,
            impl NodeCursor<Container, ObjectId<'id>>
            + Cursor<&'cursor GeneratedRustObject, ObjectId<'id>>,
        >,
    )> {
        self.cursor().get_generated::<O>(id)
    }
}

impl<'cursor, 'id, C> Cursor<&'cursor GeneratedRustObject, ObjectId<'id>>
    for GeneratedCursor<'cursor, 'id, C>
where
    C: Cursor<&'cursor GeneratedRustObject, ObjectId<'id>>,
{
    fn value(&self) -> &'cursor GeneratedRustObject {
        self.cursor.value()
    }

    fn key(&self) -> Option<&ObjectId<'id>> {
        self.cursor.key()
    }

    fn go_back(self) -> Option<Self> {
        Some(GeneratedCursor::new(self.cursor.go_back()?))
    }

    fn next(self, key: &ObjectId<'id>) -> Option<Self> {
        Some(GeneratedCursor::new(self.cursor.next(key)?))
    }
}

impl<'cursor, 'id, Key, Value, C> NodeCursor<Value, Key> for GeneratedCursor<'cursor, 'id, C>
where
    Key: Eq + Hash,
    C: NodeCursor<Value, Key> + Cursor<&'cursor GeneratedRustObject, ObjectId<'id>>,
{
    fn node(&self) -> &node::Node<Key, Value> {
        self.cursor.node()
    }
}

impl<'cursor, 'id, C> GeneratedCursor<'cursor, 'id, C>
where
    C: Cursor<&'cursor GeneratedRustObject, ObjectId<'id>>,
{
    pub fn new(cursor: C) -> Self {
        GeneratedCursor {
            cursor,
            _value: PhantomData,
            _key: PhantomData,
        }
    }

    pub fn lookup_generated<O: Object<'id>>(self, id: ObjectId<'id>) -> Option<Self> {
        O::lookup_visible(self, id)
    }

    pub fn get_generated<O: Object<'id>>(self, id: ObjectId<'id>) -> Option<(O::Generated, Self)> {
        let cursor = self.lookup_generated::<O>(id)?;
        Some((cursor.value().try_into().ok()?, cursor))
    }

    pub fn lookup_module_root(self) -> Self {
        self.lookup(|cursor, generated| {
            if generated.kind() == Kind::Module || cursor.clone().go_back().is_none() {
                LookupResult::Stop(cursor.clone())
            } else {
                LookupResult::GoBack
            }
        })
        .expect("should be impossible")
    }
}

impl Drop for NamingContext<'_, '_> {
    fn drop(&mut self) {
        // SAFETY: we are in drop so there will be no usage after
        unsafe { ManuallyDrop::take(&mut self.0) }.finish();
    }
}
