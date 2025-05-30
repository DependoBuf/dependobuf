use serde_json;

#[derive(Debug)]
pub enum ConstructorError {
    MismatchedDependencies,
}

pub type Box<T> = std::boxed::Box<T>;

pub use serde;

pub use serde_json::{from_slice, to_vec};

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
    fn deserialize<'a>(slice: &'a [u8]) -> Result<Self, DeserializeError>;
}
