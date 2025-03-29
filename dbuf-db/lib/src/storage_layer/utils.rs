use super::error::StorageError;
use super::page::PageId;
use bincode::{Decode, Encode, config};
use marble::{self, Marble};

pub const BINCODE_CONFIG: config::Configuration = config::standard();

pub fn save<T: Encode>(marble: &Marble, item: &T, id: PageId) -> Result<(), StorageError> {
    let encoded: Vec<u8> = bincode::encode_to_vec(item, BINCODE_CONFIG)?;
    marble.write_batch([(id, Some(&encoded))])?;
    Ok(())
}

pub fn load<T: Decode<()>>(marble: &Marble, id: PageId) -> Result<Option<T>, StorageError> {
    match marble.read(id)? {
        Some(data) => {
            let (item, _): (T, usize) = bincode::decode_from_slice(&data[..], BINCODE_CONFIG)?;
            Ok(Some(item))
        }
        None => Ok(None),
    }
}
