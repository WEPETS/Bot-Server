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
pub struct Bot {
    pub bot_id: String,
    pub user_id: i64,
    pub last_creation: i64,
}

#[derive(Deserialize, Fields)]
pub struct BotForCreate {
    pub bot_id: String,
    pub user_id: i64,
    pub last_creation: i64,
}
// endregion:    --- Types

pub trait BotModel: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl BotModel for Bot {}

pub struct BotBmc {}

// region:    --- Discord Profile Controller
impl DbBmc for BotBmc {
    const TABLE: &'static str = "bot";
}

impl BotBmc {
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, bot_id: String) -> Result<E>
    where
        E: BotModel,
    {
        let db_pool = mm.get_db_pool();

        let entity: E = sqlb::select()
            .table(Self::TABLE)
            .columns(E::field_names())
            .and_where("bot_id", "=", bot_id.clone())
            .fetch_optional(db_pool)
            .await?
            .ok_or(Error::EntityNotFoundString {
                entity: Self::TABLE,
                id: bot_id,
            })?;

        Ok(entity)
    }

    pub async fn create(ctx: &Ctx, mm: &ModelManager, data: BotForCreate) -> Result<String> {
        let db_pool = mm.get_db_pool();

        let fields = data.not_none_fields();
        let (id,) = sqlb::insert()
            .table(Self::TABLE)
            .data(fields)
            .returning(&["bot_id"])
            .fetch_one::<_, (String,)>(db_pool)
            .await?;

        Ok(id)
    }

    async fn delete(ctx: &Ctx, mm: &ModelManager, bot_id: String) -> Result<()> {
        let db_pool = mm.get_db_pool();

        let count = sqlb::delete()
            .table(Self::TABLE)
            .and_where("bot_id", "=", bot_id.clone())
            .exec(db_pool)
            .await?;

        if count == 0 {
            Err(Error::EntityNotFoundString {
                entity: Self::TABLE,
                id: bot_id,
            })
        } else {
            Ok(())
        }
    }
}
// endregion:    --- Discord Profile Controller

// region:    --- Tests
#[cfg(test)]
mod tests {
    use crate::{
        _dev_init,
        ctx::Ctx,
        models::{bot::Bot, wallet::WalletForUpdateFaucet, ModelManager, UserBmc, UserForCreate},
    };
    use dotenvy::dotenv;
    use serial_test::serial;

    use super::{BotBmc, BotForCreate};

    #[serial]
    #[tokio::test]
    async fn test_create_delete_bot() {
        dotenv().ok();

        let ctx = Ctx::root_ctx();
        let mm = _dev_init::init_db_for_test().await;

        let user_id = UserBmc::create(
            &ctx,
            &mm,
            UserForCreate {
                username: None,
                pwd: None,
                email: None,
            },
        )
        .await
        .unwrap();

        let bot_id = BotBmc::create(
            &ctx,
            &mm,
            BotForCreate {
                bot_id: "abc".to_string(),
                user_id,
                last_creation: 10000012331,
            },
        )
        .await
        .unwrap();

        let bot = BotBmc::get::<Bot>(&ctx, &mm, bot_id.clone()).await.unwrap();

        assert_eq!(bot.last_creation, 10000012331);

        BotBmc::delete(&ctx, &mm, bot_id).await.unwrap();
    }
}
// endregion:    --- Tests
