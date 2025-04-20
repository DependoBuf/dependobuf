#[derive(Default)]
pub struct AstConstructorsStack<'a> {
    stack: Vec<&'a str>,
}

impl<'a> AstConstructorsStack<'a> {
    pub fn new() -> AstConstructorsStack<'a> {
        AstConstructorsStack { stack: Vec::new() }
    }
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
    pub fn enter_constructor(&mut self, constructor_name: &'a str) {
        self.stack.push(constructor_name);
    }
    pub fn leave_constructor(&mut self) {
        assert!(!self.stack.is_empty());

        self.stack.pop();
    }

    pub fn get_last(&self) -> &'a str {
        match self.stack.last() {
            Some(ctr) => ctr,
            None => panic!("get last on empty constructors stack"),
        }
    }
}
