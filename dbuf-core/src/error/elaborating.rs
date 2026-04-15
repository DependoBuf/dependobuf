//! Module contains `elaborating::Error` - errors that appear
//! during elaborating phase

use std::fmt::{Display, Formatter};
use thiserror::Error;

/// TODO: implement
#[derive(Debug, Error)]
pub enum Error {
    ElaboratingError,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!("error display not implemented")
    }
}
