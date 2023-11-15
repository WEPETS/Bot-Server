// region:    --- Modules & Imports
mod error;

pub use self::error::{Error, Result};

use crate::{config, get_config};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
// endregion: --- Modules & Imports

pub type DbPool = Pool<Postgres>;

// init new database pool connection
pub async fn new_db_pool() -> Result<DbPool> {
    let max_connections = if cfg!(test) { 1 } else { 5 };

    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&get_config().SERVICE_DB_URL)
        .await
        .map_err(|ex| Error::FailToCreatePool(ex.to_string()))
}
