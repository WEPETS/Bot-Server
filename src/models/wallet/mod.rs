// region:    --- Imports
use super::base_crud::{update, DbBmc};
use super::{base_crud, ModelManager};
use crate::ctx::Ctx;
use crate::get_config;
use crate::models::error::{Error, Result};
use crate::pwd::{self, ContentToHash};
use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use tracing::info;
use uuid::Uuid;
// endregion:    --- Imports

// region:    --- Types
#[derive(Clone, Fields, FromRow, Debug, Serialize)]
pub struct Wallet {
    pub id: i64,
    pub pub_key: String,
    pub sign_type: String,
}

#[derive(Deserialize, Fields)]
pub struct WalletForCreate {
    pub id: i64,
    pub pub_key: String,
    pub sign_type: String,
    pub phrase: String,
}
// endregion:    --- Types

pub trait WalletModel: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl WalletModel for Wallet {}

pub struct WalletBmc {}

// region:    --- Discord Profile Controller
impl DbBmc for WalletBmc {
    const TABLE: &'static str = "wallet";
}

impl WalletBmc {
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where
        E: WalletModel,
    {
        base_crud::get::<Self, E>(ctx, mm, id).await
    }

    pub async fn create(ctx: &Ctx, mm: &ModelManager, data: WalletForCreate) -> Result<i64> {
        base_crud::create::<WalletBmc, WalletForCreate>(ctx, mm, data).await
    }
}
// endregion:    --- Discord Profile Controller
