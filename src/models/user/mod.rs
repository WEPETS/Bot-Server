use crate::ctx::Ctx;
use crate::get_config;
// use crate::pwd::{self, ContentToHash};
use crate::models::error::{Error, Result};
use crate::pwd::{self, ContentToHash};
use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use tracing::info;
use uuid::Uuid;

use super::base_crud::{update, DbBmc};
use super::{base_crud, ModelManager};

// region:    --- User Types
#[derive(Clone, Fields, FromRow, Debug, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Clone, Fields, FromRow, Debug)]
pub struct UserForLogin {
    pub id: i64,
    pub username: String,

    // -- pwd and token info
    pub pwd: String,
    pub token_salt: Uuid,
}

#[derive(Clone, Fields, FromRow, Debug)]
pub struct UserForAuth {
    pub id: i64,
    pub username: String,

    pub token_salt: Uuid,
}

#[derive(Deserialize, Fields)]
pub struct UserForCreate {
    pub username: String,
    pub pwd: String,
}

#[derive(Deserialize, Fields)]
pub struct UserForUpdate {
    pwd: String,
}
/// Marker trait
pub trait UserModel: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserModel for User {}
impl UserModel for UserForLogin {}
impl UserModel for UserForAuth {}
// endregion: --- User Types
pub struct UserBmc {}

// region:    --- User Controller
impl DbBmc for UserBmc {
    const TABLE: &'static str = "user";
}

impl UserBmc {
    pub async fn list<E>(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
    where
        E: UserModel,
    {
        base_crud::list::<Self, E>(ctx, mm).await
    }

    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where
        E: UserModel,
    {
        base_crud::get::<Self, E>(ctx, mm, id).await
    }

    pub async fn get_first_by_username<E>(
        ctx: &Ctx,
        mm: &ModelManager,
        username: &str,
    ) -> Result<Option<E>>
    where
        E: UserModel,
    {
        let db_pool = mm.get_db_pool();

        let user = sqlb::select()
            .table(Self::TABLE)
            .and_where("username", "=", username)
            .fetch_optional::<_, E>(db_pool)
            .await?;

        Ok(user)
    }

    pub async fn create(ctx: &Ctx, mm: &ModelManager, data: UserForCreate) -> Result<i64> {
        let config = get_config();

        let clear_pwd = data.pwd.clone();
        let mut data_to_create = data;

        data_to_create.pwd = pwd::hash_pwd(&ContentToHash {
            content: clear_pwd,
            salt: config.SERVICE_PASSWORD_SALT.to_string(),
        })?;

        base_crud::create::<UserBmc, UserForCreate>(ctx, mm, data_to_create).await
    }

    pub async fn update(ctx: &Ctx, mm: &ModelManager, id: i64, data: UserForUpdate) -> Result<()> {
        let config = get_config();

        let hashed_pwd = pwd::hash_pwd(&ContentToHash {
            content: data.pwd.to_string(),
            salt: config.SERVICE_PASSWORD_SALT.to_string(),
        })?;

        info!(hashed_pwd);

        let pwd = UserForUpdate { pwd: hashed_pwd };
        base_crud::update::<UserBmc, UserForUpdate>(ctx, mm, id, pwd).await
    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base_crud::delete::<UserBmc>(ctx, mm, id).await
    }
}
// endregion:    --- User Controller

// region:    --- Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::_dev_init;
    use crate::models::Error;
    use anyhow::Result;
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_create_update_ok() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_init::init_db_for_test().await;
        let ctx = Ctx::root_ctx();

        let user_c = UserForCreate {
            username: "abc".to_string(),
            pwd: "123".to_string(),
        };

        let id = UserBmc::create(&ctx, &mm, user_c).await?;

        let user = UserBmc::get::<UserForLogin>(&ctx, &mm, id).await?;

        assert_eq!(user.pwd, "123".to_string());

        UserBmc::update(
            &ctx,
            &mm,
            id,
            UserForUpdate {
                pwd: "new_pwd".to_string(),
            },
        )
        .await?;

        let user = UserBmc::get::<UserForLogin>(&ctx, &mm, id).await?;

        assert_eq!(user.pwd, "new_pwd".to_string());

        UserBmc::delete(&ctx, &mm, id).await?;

        let user = UserBmc::get::<UserForLogin>(&ctx, &mm, id).await;

        assert!(
            matches!(user, Err(Error::EntityNotFound { entity: "user", id })),
            "EntityNotFound not matching"
        );

        Ok(())
    }
}
// endregion    --- Tests
