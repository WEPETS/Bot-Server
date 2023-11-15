use crate::models;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // -- Config
    ConfigMissing(&'static str),

    WrongFormat(&'static str),

    Model(models::Error),
}

// region:    --- Froms
impl From<models::Error> for Error {
    fn from(val: models::Error) -> Self {
        Self::Model(val)
    }
}
// endregion: --- Froms

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion:    --- Error Boilerplate
