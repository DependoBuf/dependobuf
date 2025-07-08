use std::{
    any::Any,
    marker::PhantomData,
    ptr,
    rc::{Rc, Weak},
};

// just marker trait
pub trait Node {}

/// This enum is expected to used in order to find semantically equal object comparing its `ObjectIds`.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum NodeIdStorage<'a> {
    Pointer {
        pointer: usize,
        _m: PhantomData<&'a dyn Any>,
    },
    // TODO: because of that ObjectId could not be cheaply copied, maybe change this to Cow
    Owned {
        name: String,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct NodeId<'a>(NodeIdStorage<'a>);

impl NodeId<'_> {
    pub fn owned(name: String) -> NodeId<'static> {
        NodeId(NodeIdStorage::Owned { name })
    }

    // dirty but needed because references do not point to some arena
    pub fn id<T: Node>(this: &T) -> NodeId<'static> {
        NodeId(NodeIdStorage::Pointer {
            pointer: ptr::from_ref(this) as usize,
            _m: PhantomData,
        })
    }

    pub fn id_rc<T: 'static + Node>(this: &Rc<T>) -> NodeId<'static> {
        NodeId(NodeIdStorage::Pointer {
            pointer: Rc::as_ptr(this) as usize,
            _m: PhantomData,
        })
    }

    pub fn id_weak<T: 'static + Node>(this: &Weak<T>) -> NodeId<'static> {
        let _ = this.upgrade().expect("dandling weak does not have id");
        NodeId(NodeIdStorage::Pointer {
            pointer: this.as_ptr() as usize,
            _m: PhantomData,
        })
    }
}
