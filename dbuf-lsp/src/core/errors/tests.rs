use std::collections::HashSet;

use strum::IntoEnumIterator;

use super::FormatError;
use super::RenameError;

trait ErrorsEnum {
    fn get_code(self) -> i64;
}

impl ErrorsEnum for FormatError {
    fn get_code(self) -> i64 {
        self.to_jsonrpc_error::<()>().unwrap_err().code.code()
    }
}

impl ErrorsEnum for RenameError {
    fn get_code(self) -> i64 {
        self.to_jsonrpc_error::<()>().unwrap_err().code.code()
    }
}

#[test]
fn test_unique_codes() {
    let mut codes = HashSet::new();

    FormatError::iter()
        .map(ErrorsEnum::get_code)
        .chain(RenameError::iter().map(ErrorsEnum::get_code))
        .for_each(|c| {
            if codes.get(&c).is_some() {
                panic!("dublicate code: {c}");
            }
            codes.insert(c);
        });
}
