//! Module provides AstConstructorStack - simple
//! stack for constructor calls.
//!

/// Simple stack for constructor calls.
///
/// Usage:
/// * call `enter_constructor` on entering in constructor call.
/// * call `leave_constructor` on leaving from constructor call.
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
    /// Entering in constructor call.
    pub fn enter_constructor(&mut self, constructor_name: &'a str) {
        self.stack.push(constructor_name);
    }
    /// Leaving from constructor call.
    ///
    /// Panics if there is no constructor calls.
    pub fn leave_constructor(&mut self) {
        assert!(!self.stack.is_empty());

        self.stack.pop();
    }

    /// Returns last constructor call.
    ///
    /// Panics if there is no constructor calls.
    #[allow(dead_code, reason = "not using yet")]
    pub fn get_last(&self) -> &'a str {
        self.stack
            .last()
            .map_or_else(|| panic!("get last on empty constructor stack"), |ctr| ctr)
    }
}
