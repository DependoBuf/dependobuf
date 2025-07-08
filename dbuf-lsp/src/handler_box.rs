use std::ops::Deref;
use std::sync::OnceLock;

use tower_lsp::lsp_types::InitializeParams;

/// Box for all handlers.
///
/// Each handler should implement
/// trait Handler with function create,
/// that returns Capabilities and instance
/// of handler.
pub struct HandlerBox<T> {
    handler: OnceLock<T>,
}

pub trait Handler {
    type Capabilities;
    fn create(init: &InitializeParams) -> (Self::Capabilities, Self);
}

impl<T: Handler> HandlerBox<T> {
    /// # Panics
    ///
    /// Will panic if called more than once.
    pub fn init(&self, init: &InitializeParams) -> T::Capabilities {
        let (capabilities, state) = T::create(init);
        let res = self.handler.set(state);
        assert!(res.is_ok(), "init should be called once");
        capabilities
    }
}

impl<T> Deref for HandlerBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.handler.get().expect("handler should be initialized")
    }
}

impl<T> Default for HandlerBox<T> {
    fn default() -> Self {
        Self {
            handler: OnceLock::default(),
        }
    }
}
