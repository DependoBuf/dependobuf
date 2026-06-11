use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Context<'a, Key, Value, Alias>
where
    Key: Hash + Eq + 'a,
    Value: 'a,
    Alias: 'a,
{
    parent: Option<&'a Context<'a, Key, Value, Alias>>,
    pub terms: HashMap<Key, Value>,
    pub aliases: HashMap<Key, Alias>,
}

impl<'a, Key, Value, Alias> Default for Context<'a, Key, Value, Alias>
where
    Key: Hash + Eq + 'a,
    Value: 'a,
    Alias: 'a,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Key, Value, Alias> Context<'a, Key, Value, Alias>
where
    Key: Hash + Eq + 'a,
    Value: 'a,
    Alias: 'a,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            parent: None,
            terms: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    #[must_use]
    pub fn new_layer(&'a self) -> Self {
        Context {
            parent: Some(self),
            terms: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    pub fn get(&self, var_name: &Key) -> Option<&Value> {
        self.terms
            .get(var_name)
            .or_else(|| self.parent.and_then(|p| p.get(var_name)))
    }

    pub fn insert(&mut self, name: Key, ty: Value) {
        self.terms.insert(name, ty);
    }

    pub fn get_alias(&self, name: &Key) -> Option<&Alias> {
        self.aliases
            .get(name)
            .or_else(|| self.parent.and_then(|p| p.get_alias(name)))
    }

    pub fn insert_alias(&mut self, name: Key, alias: Alias) {
        self.aliases.insert(name, alias);
    }

    pub fn lookup(&self, name: &Key) -> Option<Lookup<&Value, &Alias>> {
        if let Some(alias) = self.aliases.get(name) {
            return Some(Lookup::Alias(alias));
        }
        if let Some(term) = self.terms.get(name) {
            return Some(Lookup::Term(term));
        }
        self.parent.and_then(|p| p.lookup(name))
    }
}

pub enum Lookup<V, A> {
    Term(V),
    Alias(A),
}
