//! The module helps managing strings. Exports:
//! * `InternedString` structure, that allows efficient string comparison.
//!
use internment::ArcIntern;

/// Interned String struct stores Arc pointer to string.
/// Allows fast comparison and cloning.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct InternedString {
    inner: ArcIntern<String>,
}

impl From<String> for InternedString {
    fn from(value: String) -> Self {
        Self {
            inner: ArcIntern::new(value),
        }
    }
}

impl AsRef<str> for InternedString {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}
