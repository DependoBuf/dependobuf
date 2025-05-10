use std::{
    collections::{HashMap, hash_map},
    fmt::Debug,
    hash::Hash,
};

/// HashMap wrapper that supports natural for the scopes nesting
#[derive(Debug)]
pub struct Scope<'a, Key, Value>
where
    Key: Hash + Eq + 'a,
    Value: 'a,
{
    parent: Option<&'a Scope<'a, Key, Value>>,
    map: HashMap<Key, Value>,
}

impl<'a, Key, Value> Scope<'a, Key, Value>
where
    Key: Hash + Eq,
{
    pub fn empty() -> Self {
        Scope {
            parent: None,
            map: HashMap::new(),
        }
    }

    pub fn nested_in(parent: &'a Scope<'a, Key, Value>) -> Self {
        Scope {
            parent: Some(parent),
            map: HashMap::new(),
        }
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.get(key).is_some()
    }

    pub fn get(&self, key: &Key) -> Option<&Value> {
        self.map
            .get(key)
            .or(self.parent.and_then(|parent| parent.get(key)))
    }

    pub fn try_insert(&mut self, key: Key, value: Value) -> bool {
        if self.contains(&key) {
            false
        } else {
            self.map.insert(key, value);
            true
        }
    }

    pub fn flat_iter(&'a self) -> FlatIter<'a, Key, Value> {
        FlatIter {
            iter: self.map.iter(),
        }
    }

    pub fn flat_into_iter(self) -> FlatIntoIter<Key, Value> {
        FlatIntoIter {
            iter: self.map.into_iter(),
        }
    }

    pub fn recurse_iter(&'a self) -> RecurseIter<'a, Key, Value> {
        RecurseIter {
            current_scope: self,
            iter: self.map.iter(),
        }
    }
}

pub struct FlatIter<'a, Key, Value>
where
    Key: Hash + Eq,
{
    iter: hash_map::Iter<'a, Key, Value>,
}

impl<'a, Key, Value> Iterator for FlatIter<'a, Key, Value>
where
    Key: Hash + Eq,
{
    type Item = (&'a Key, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct FlatIntoIter<Key, Value>
where
    Key: Hash + Eq,
{
    iter: hash_map::IntoIter<Key, Value>,
}

impl<Key, Value> Iterator for FlatIntoIter<Key, Value>
where
    Key: Hash + Eq,
{
    type Item = (Key, Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct RecurseIter<'a, Key, Value>
where
    Key: Hash + Eq,
{
    current_scope: &'a Scope<'a, Key, Value>,
    iter: hash_map::Iter<'a, Key, Value>,
}

impl<'a, Key, Value> Iterator for RecurseIter<'a, Key, Value>
where
    Key: Hash + Eq,
{
    type Item = (&'a Key, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(value) = self.iter.next() {
                return Some(value);
            }
            if let Some(parent) = self.current_scope.parent {
                self.current_scope = parent;
                self.iter = self.current_scope.map.iter();
            } else {
                return None;
            }
        }
    }
}
