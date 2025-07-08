#[derive(Debug)]
pub enum ConstructorError {
    MismatchedDependencies,
}

pub type Box<T> = std::boxed::Box<T>;

pub use serde;

pub use serde_json::{from_slice, to_vec};

#[allow(dead_code, reason = "Deserialization is not ready")]
pub struct DeserializeError(serde_json::Error);

impl From<serde_json::Error> for DeserializeError {
    fn from(value: serde_json::Error) -> Self {
        DeserializeError(value)
    }
}

pub trait Serialize {
    fn serialize(self) -> Box<[u8]>;
}

pub trait Deserialize: Sized {
    /// # Errors
    ///
    /// TODO: explain when `DeserializeError` is returned.
    fn deserialize(slice: &[u8]) -> Result<Self, DeserializeError>;
}
