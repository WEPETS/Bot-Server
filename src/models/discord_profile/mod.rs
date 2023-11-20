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
pub struct DiscordProfile {
    pub id: i64,
    pub discord_id: i64,
    pub username: String,
    pub global_name: String,
    pub avatar: String,
}

#[derive(Deserialize, Fields)]
pub struct DiscordProfileForCreate {
    pub id: i64,
    pub discord_id: i64,
    pub username: String,
    pub global_name: String,
    pub avatar: String,
}
// endregion:    --- Types

pub trait DiscordProfileModel: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl DiscordProfileModel for DiscordProfile {}

pub struct DiscordProfileBmc {}

// region:    --- Discord Profile Controller
impl DbBmc for DiscordProfileBmc {
    const TABLE: &'static str = "discord_profile";
}

impl DiscordProfileBmc {
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where
        E: DiscordProfileModel,
    {
        base_crud::get::<Self, E>(ctx, mm, id).await
    }

    pub async fn get_by_discord_id<E>(ctx: &Ctx, mm: &ModelManager, discord_id: i64) -> Result<E>
    where
        E: DiscordProfileModel,
    {
        let db_pool = mm.get_db_pool();

        let entity: E = sqlb::select()
            .table(DiscordProfileBmc::TABLE)
            .columns(E::field_names())
            .and_where("discord_id", "=", discord_id)
            .fetch_optional(db_pool)
            .await?
            .ok_or(Error::EntityNotFound {
                entity: DiscordProfileBmc::TABLE,
                id: discord_id,
            })?;

        Ok(entity)
    }

    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        data: DiscordProfileForCreate,
    ) -> Result<i64> {
        base_crud::create::<DiscordProfileBmc, DiscordProfileForCreate>(ctx, mm, data).await
    }
}
// endregion:    --- Discord Profile Controller
