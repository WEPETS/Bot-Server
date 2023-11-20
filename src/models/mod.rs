// region -- Modules
mod base_crud;
pub mod discord_profile;
mod error;
pub mod user;
pub mod wallet;

pub use self::error::{Error, Result};
use crate::store::{new_db_pool, DbPool};
pub use user::{User, UserBmc, UserForAuth, UserForCreate, UserForLogin, UserForUpdate, UserModel};
// endregion -- Modules

#[derive(Clone)]
pub struct ModelManager {
    pub db_pool: DbPool,
}

impl ModelManager {
    /// Constructor
    pub async fn new() -> Result<Self> {
        let db_pool = new_db_pool().await?;

        Ok(ModelManager { db_pool })
    }

    /// Returns the sqlx db pool reference.
    /// (Only for the model layer)
    pub(in crate::models) fn get_db_pool(&self) -> &DbPool {
        &self.db_pool
    }
}
