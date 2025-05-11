use std::ops::Deref;
use std::sync::OnceLock;

/// Box for all handlers.
///
/// Each handler should implement
/// HandlerBox<Handler> with method init, which:
/// * uses HandlerBox.set() to init state.
/// * returns capabilities of handler.
pub struct HandlerBox<T> {
    handler: OnceLock<T>,
}

impl<T> HandlerBox<T> {
    pub(crate) fn set(&self, state: T) {
        let res = self.handler.set(state);
        assert!(res.is_ok(), "set should be called once");
    }
}

impl<T> Deref for HandlerBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.handler.get().expect("handler should be initialized")
    }
}

// Strange: cannot use #[derive(Default)] (main can't use action_handler: Default::default()). Seem like a bug in Rust for me,
// so any explanatory comment whould be helpful.
impl<T> Default for HandlerBox<T> {
    fn default() -> Self {
        Self {
            handler: Default::default(),
        }
    }
}
