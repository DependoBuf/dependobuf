use std::collections::HashSet;

use core::convert::Into;

use strum::IntoEnumIterator;

use super::Error;
use super::FormatError;
use super::RenameError;

#[test]
fn test_unique_codes() {
    let mut codes = HashSet::new();

    FormatError::iter()
        .map(Into::into)
        .chain(RenameError::iter().map(Into::into))
        .for_each(|e: Error| {
            let c = e.get_code();
            if codes.get(&c).is_some() {
                panic!("dublicate code: {c}");
            }
            codes.insert(c);
        });
}
