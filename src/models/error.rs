use crate::{pwd, store};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
    EntityNotFound { entity: &'static str, id: i64 },
    EntityNotFoundString { entity: &'static str, id: String },

    // -- Modules
    Pwd(pwd::Error),
    Store(store::Error),

    // -- Externals
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
}

// region:    --- Froms
impl From<pwd::Error> for Error {
    fn from(val: pwd::Error) -> Self {
        Self::Pwd(val)
    }
}

impl From<store::Error> for Error {
    fn from(val: store::Error) -> Self {
        Self::Store(val)
    }
}

impl From<sqlx::Error> for Error {
    fn from(val: sqlx::Error) -> Self {
        Self::Sqlx(val)
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
// endregion: --- Error Boilerplate
