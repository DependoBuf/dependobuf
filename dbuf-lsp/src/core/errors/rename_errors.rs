use strum::IntoDiscriminant;
use strum_macros::{EnumDiscriminants, EnumIter};

use thiserror::Error;

#[derive(EnumIter, EnumDiscriminants, Error, Debug)]
pub enum RenameError {
    #[error("rename to empty string is forbidden")]
    ToEmpty,
    #[error("rename to builtin type is forbidden")]
    ToBuiltin,
    #[error("rename to keyword is forbidden")]
    ToKeyword,
    #[error("builtin type can't be renamed")]
    OfBuiltin,
    #[error("rename to previous name is useless")]
    ToPrevious,
    #[error("none symbol can't be renamed")]
    OfNone,
    #[error("'{0}' is not correct type name")]
    ToBadType(String),
    #[error("'{0}' is not correct constructor name")]
    ToBadConstructor(String),
    #[error("'{0}' is not correct dependency name")]
    ToBadDependency(String),
    #[error("'{0}' is not correct alias name")]
    ToBadAlias(String),
    #[error("'{0}' is not correct field name")]
    ToBadField(String),
    #[error("constructor or type '{0}' exists")]
    ToExistingType(String),
    #[error("constructor or type '{t}' already contains '{r}'")]
    ToExistingResource { t: String, r: String },
}

impl RenameError {
    pub fn get_code(&self) -> i64 {
        self.discriminant() as i64
    }
}
