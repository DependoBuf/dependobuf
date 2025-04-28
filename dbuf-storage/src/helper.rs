use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn create_alphanumeric_hash(input: &str) -> String {
    let mut hasher = DefaultHasher::new();

    input.hash(&mut hasher);

    let hash_value = hasher.finish();

    const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    const BASE: u64 = 62;

    let mut result = String::new();
    let mut remainder = hash_value;

    if remainder == 0 {
        return "0".to_string();
    }

    while remainder > 0 {
        let index = (remainder % BASE) as usize;
        result.insert(0, CHARSET[index] as char);
        remainder /= BASE;
    }

    result
}

pub fn get_json_hash(input: &str) -> String {
    create_alphanumeric_hash(input)
}
