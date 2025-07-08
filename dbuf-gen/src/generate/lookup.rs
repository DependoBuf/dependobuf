use std::{hash::Hash, marker::PhantomData};

use super::node::Node;

#[allow(dead_code, reason = "??? (GoNext never constructed)")]
pub enum LookupResult<Key, Result> {
    GoBack,
    GoNext(Key),
    Stop(Result),
}

pub trait NodeCursor<Value, Key>: Sized + Clone {
    fn node(&self) -> &Node<Key, Value>;
}

pub trait Cursor<Value, Key>: Sized + Clone {
    fn value(&self) -> Value;

    fn key(&self) -> Option<&Key>;

    fn go_back(self) -> Option<Self>;

    fn next(self, key: &Key) -> Option<Self>;

    fn lookup<Result, F: FnMut(&Self, Value) -> LookupResult<Key, Result>>(
        mut self,
        mut f: F,
    ) -> Option<Result> {
        let result = loop {
            match f(&self, self.value()) {
                LookupResult::GoBack => self = self.go_back()?,
                LookupResult::GoNext(id) => self = self.next(&id)?,
                LookupResult::Stop(result) => break result,
            }
        };

        Some(result)
    }

    fn value_map<NewValue, F>(self, f: F) -> CursorValueMap<Value, Self, F>
    where
        F: Fn(Value) -> NewValue,
    {
        CursorValueMap {
            cursor: self,
            mapper: f,
            _marker: PhantomData,
        }
    }

    fn apply<F: FnMut(&Self, Value)>(self, mut f: F) -> Self {
        f(&self, self.value());
        self
    }
}

#[derive(Clone, Debug)]
pub struct CursorValueMap<OrigValue, C, F> {
    cursor: C,
    mapper: F,
    _marker: PhantomData<OrigValue>,
}

impl<OrigValue, ResultValue, Key, C, F> Cursor<ResultValue, Key> for CursorValueMap<OrigValue, C, F>
where
    C: Cursor<OrigValue, Key>,
    F: Fn(OrigValue) -> ResultValue,
    OrigValue: Clone,
    F: Clone,
{
    fn value(&self) -> ResultValue {
        (self.mapper)(self.cursor.value())
    }

    fn key(&self) -> Option<&Key> {
        self.cursor.key()
    }

    fn go_back(self) -> Option<Self> {
        Some(CursorValueMap {
            cursor: self.cursor.go_back()?,
            mapper: self.mapper,
            _marker: PhantomData,
        })
    }

    fn next(self, key: &Key) -> Option<Self> {
        Some(CursorValueMap {
            cursor: self.cursor.next(key)?,
            mapper: self.mapper,
            _marker: PhantomData,
        })
    }
}

impl<Value, OrigValue, Key, C, F> NodeCursor<Value, Key> for CursorValueMap<OrigValue, C, F>
where
    OrigValue: Clone,
    Key: Eq + Hash,
    C: NodeCursor<Value, Key>,
    F: Clone,
{
    fn node(&self) -> &Node<Key, Value> {
        self.cursor.node()
    }
}
