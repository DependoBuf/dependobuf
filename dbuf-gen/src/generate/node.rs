use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone)]
pub struct Node<Key, Value> {
    pub(super) nested: HashMap<Key, Self>,
    pub(super) detail: Value,
}

#[allow(dead_code, reason = "??? (some methods are never used)")]
impl<Key: Eq + Hash, Value> Node<Key, Value> {
    pub(super) fn new(detail: Value) -> Self {
        Node {
            nested: HashMap::new(),
            detail,
        }
    }

    pub(super) fn try_insert(&mut self, key: Key, node: Self) -> bool {
        if let std::collections::hash_map::Entry::Vacant(e) = self.nested.entry(key) {
            e.insert(node);
            true
        } else {
            false
        }
    }

    pub(super) fn insert(&mut self, key: Key, node: Self) {
        assert!(self.try_insert(key, node), "could not insert");
    }

    pub(super) fn remove(&mut self, key: &Key) -> Option<Self> {
        self.nested.remove(key)
    }

    pub(super) fn detail(&self) -> &Value {
        &self.detail
    }

    pub(super) fn next(&self, key: &Key) -> Option<&Node<Key, Value>> {
        self.nested.get(key)
    }

    pub(super) fn next_with_key(&self, key: &Key) -> Option<(&Key, &Node<Key, Value>)> {
        self.nested.get_key_value(key)
    }
}
