use sui_sdk::error::Error as SuiError;
use sui_sdk::types::base_types::ObjectIDParseError;

use crate::models;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    TransactionFail,
}

// region:    --- Froms
// impl From<SuiError> for Error {
//     fn from(val: SuiError) -> Self {
//         Self::Sui(val)
//     }
// }
// endregion: --- Froms

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion:    --- Error Boilerplate
