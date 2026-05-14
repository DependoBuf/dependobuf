use crate::DeserializeError::LiteralError;
use std::io;
use std::io::{Read, Write};

#[derive(Debug)]
pub enum ConstructorError {
    MismatchedDependencies,
}

#[derive(Debug)]
pub enum DeserializeError {
    IoError(io::Error),
    ConstructorError(ConstructorError),
    UnknownDescriptor,
    DependenciesDescriptorMismatch,
    LiteralError(String),
}

pub type Box<T> = std::boxed::Box<T>;

pub trait DbufPrimitive: Sized {
    /// Serialize method for primitive types
    ///
    /// # Errors
    ///  Returns an I/O error if the `write_all` method on the serialized data throws an error.
    fn dbuf_serialize<W: Write>(&self, writer: &mut W) -> io::Result<()>;

    /// Deserialize method for primitive types
    ///
    /// # Errors
    ///  * `DeserializeError::IoError` when `read_exact` method on the buffer throws an error.
    ///  * `LiteralError` when literal parsing fails.
    fn dbuf_deserialize<R: Read>(reader: &mut R) -> Result<Self, DeserializeError>;
}

impl DbufPrimitive for bool {
    fn dbuf_serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[u8::from(*self)])
    }
    fn dbuf_deserialize<R: Read>(reader: &mut R) -> Result<Self, DeserializeError> {
        let mut buf = [0u8; 1];
        reader
            .read_exact(&mut buf)
            .map_err(DeserializeError::IoError)?;
        Ok(buf[0] != 0)
    }
}

impl DbufPrimitive for i64 {
    fn dbuf_serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
    fn dbuf_deserialize<R: Read>(reader: &mut R) -> Result<Self, DeserializeError> {
        let mut buf = [0u8; 8];
        reader
            .read_exact(&mut buf)
            .map_err(DeserializeError::IoError)?;
        Ok(i64::from_le_bytes(buf))
    }
}

impl DbufPrimitive for u64 {
    fn dbuf_serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
    fn dbuf_deserialize<R: Read>(reader: &mut R) -> Result<Self, DeserializeError> {
        let mut buf = [0u8; 8];
        reader
            .read_exact(&mut buf)
            .map_err(DeserializeError::IoError)?;
        Ok(u64::from_le_bytes(buf))
    }
}

impl DbufPrimitive for f64 {
    fn dbuf_serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
    fn dbuf_deserialize<R: Read>(reader: &mut R) -> Result<Self, DeserializeError> {
        let mut buf = [0u8; 8];
        reader
            .read_exact(&mut buf)
            .map_err(DeserializeError::IoError)?;
        Ok(f64::from_le_bytes(buf))
    }
}

impl DbufPrimitive for String {
    fn dbuf_serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let len = self.len() as u64;
        writer.write_all(&len.to_le_bytes())?;
        writer.write_all(self.as_bytes())
    }
    fn dbuf_deserialize<R: Read>(reader: &mut R) -> Result<Self, DeserializeError> {
        let mut len_buf = [0u8; 8];
        reader
            .read_exact(&mut len_buf)
            .map_err(DeserializeError::IoError)?;
        let len = usize::try_from(u64::from_le_bytes(len_buf))
            .map_err(|_| LiteralError("Line length too long".to_string()))?;
        let mut bytes = vec![0u8; len];
        reader
            .read_exact(&mut bytes)
            .map_err(DeserializeError::IoError)?;
        String::from_utf8(bytes)
            .map_err(|_| LiteralError("Invalid UTF-8 sequence in string".to_string()))
    }
}
