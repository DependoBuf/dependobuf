use super::codegen::Tag;
use std::{
    collections::{
        hash_map::{self, Entry},
        HashMap,
    },
    hash::Hash,
};

/// HashMap wrapper that supports natural for the scopes nesting
pub struct Scope<'a, Key, Value>
where
    Key: Hash + Eq,
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

struct FlatIter<'a, Key, Value>
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

struct FlatIntoIter<Key, Value>
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

struct RecurseIter<'a, Key, Value>
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

/// TODO: Update doc
/// 
/// This scope is used to track variables with the same name and give them additional unique tag in the form u32
/// which will be used later by formatter in order to generate unique name.
///
/// The reason why this requires special scope wrapper is following:
/// scope can only provide some value (of distinctive type) that we can afterwards use in some identifier formatter
/// to create unique identifier name. But we need to update distinctive.
///
/// So, NamingScope responsible for given identifier formatter unique tag.
pub struct TaggerScope<'a, Value>
where
    Value: Hash + Eq,
{
    scope: Scope<'a, Value, Tag>,
}

impl<'a, Value> TaggerScope<'a, Value>
where
    Value: Hash + Eq,
{
    pub fn empty() -> Self {
        TaggerScope {
            scope: Scope::empty(),
        }
    }

    pub fn nested_in(parent: &'a TaggerScope<'a, Value>) -> Self {
        TaggerScope {
            scope: Scope::nested_in(&parent.scope),
        }
    }

    pub fn contains(&self, key: &Value) -> bool {
        self.scope.contains(key)
    }

    pub fn try_insert(&mut self, key: Value) -> bool {
        self.scope.try_insert(key, Tag::default())
    }

    pub fn insert(&mut self, key: Value) -> Tag {
        match self.scope.map.entry(key) {
            hash_map::Entry::Occupied(mut occupied) => {
                let tag = occupied.get().inc();
                occupied.insert(tag.clone());
                tag
            }
            hash_map::Entry::Vacant(vacant) => {
                let tag = self
                    .scope
                    .parent
                    .and_then(|parent| parent.get(vacant.key()))
                    .cloned()
                    .map(|tag| tag.inc())
                    .unwrap_or_default();
                vacant.insert(tag.clone());
                tag
            }
        }
    }
}

// struct NamerScope
//
// pub fn name_and_add(&mut self, base: &str) -> String {
//     let suffix = match self.used_names.entry(base.to_owned()) {
//         Entry::Occupied(mut occupied) => {
//             *occupied.get_mut() += 1;
//             *occupied.get()
//         }
//         Entry::Vacant(vacant) => {
//             let new_suffix = match self.parent.and_then(|parent| parent.last_used_suffix(base))
//             {
//                 Some(suffix) => suffix + 1,
//                 None => 0,
//             };
//             vacant.insert(new_suffix);
//             new_suffix
//         }
//     };
//     Self::format(base, suffix)
// }

// fn format(base: &str, suffix: u32) -> String {
//     match suffix {
//         0 => base.to_owned(),
//         _ => format!("{}_{}", base, suffix),
//     }
// }
