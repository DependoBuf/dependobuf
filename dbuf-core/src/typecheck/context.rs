use std::collections::{HashMap};
use std::hash::Hash;

#[derive(Default, Debug, Clone)]
pub struct Context<'a, Key, Value>
where
    Key: Hash + Eq + 'a,
    Value: 'a,
{
    pub parent: Option<&'a Context<'a, Key, Value>>,
    pub terms: HashMap<Key, Value>,
}

impl<'a, Key, Value> Context<'a, Key, Value>
where
    Key: Hash + Eq + 'a,
{
    pub fn new() -> Self {
        Self {
            parent: None,
            terms: HashMap::new(),
        }
    }

    pub fn new_layer(self: &'a Self) -> Self {
        Context::<'a, Key, Value> {
            parent: Some(self),
            terms: HashMap::new(),
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
}

#[test]
pub fn test_context_find_type() {
    let mut context = Context::new();
    context.insert(String::from("a"), "A");
    context.insert(String::from("b"), "B");

    let mut new_context = context.new_layer();

    new_context.insert(String::from("c"), "C");

    let success_case: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();

    let fail_case: Vec<String> = ["d", "f"].iter().map(|s| s.to_string()).collect();

    for var_name in success_case.iter() {
        assert_ne!(new_context.get(&String::from(var_name)), None);
    }

    for var_name in fail_case.iter() {
        assert_eq!(new_context.get(&String::from(var_name)), None);
    }
}
