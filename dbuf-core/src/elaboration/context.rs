use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Context<'a, Key, Value, Alias>
where
    Key: Hash + Eq + 'a,
    Value: 'a,
    Alias: 'a,
{
    pub parent: Option<&'a Context<'a, Key, Value, Alias>>,
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

    pub fn insert(&mut self, var_name: Key, var_type: Value) {
        self.terms.insert(var_name, var_type);
    }

    pub fn get_alias(&self, name: &Key) -> Option<&Alias> {
        self.aliases
            .get(name)
            .or_else(|| self.parent.and_then(|p| p.get_alias(name)))
    }

    pub fn insert_alias(&mut self, name: Key, alias: Alias) {
        self.aliases.insert(name, alias);
    }
}

#[test]
fn test_context_find_type() {
    let mut context: Context<String, &str, ()> = Context::new();
    context.insert(String::from("a"), "A");
    context.insert(String::from("b"), "B");

    let mut new_context = context.new_layer();

    new_context.insert(String::from("c"), "C");

    let success_case: Vec<String> = ["a", "b", "c"].iter().map(ToString::to_string).collect();

    let fail_case: Vec<String> = ["d", "f"].iter().map(ToString::to_string).collect();

    for var_name in &success_case {
        assert_ne!(new_context.get(&String::from(var_name)), None);
    }

    for var_name in &fail_case {
        assert_eq!(new_context.get(&String::from(var_name)), None);
    }
}
