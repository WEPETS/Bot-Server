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
    pub last_faucet: Option<i64>,
}

#[derive(Deserialize, Fields)]
pub struct WalletForCreate {
    pub id: i64,
    pub pub_key: String,
    pub sign_type: String,
    pub phrase: String,
    pub last_faucet: Option<i64>,
}

#[derive(Deserialize, Fields)]
pub struct WalletForUpdateFaucet {
    pub last_faucet: i64,
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

    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        data: WalletForUpdateFaucet,
    ) -> Result<()> {
        base_crud::update::<WalletBmc, WalletForUpdateFaucet>(ctx, mm, id, data).await
    }

    async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base_crud::delete::<WalletBmc>(ctx, mm, id).await
    }
}
// endregion:    --- Discord Profile Controller

// region:    --- Tests
#[cfg(test)]
mod tests {
    use crate::{
        _dev_init,
        ctx::Ctx,
        models::{wallet::WalletForUpdateFaucet, ModelManager, UserBmc, UserForCreate},
    };
    use dotenvy::dotenv;
    use serial_test::serial;

    use super::{Wallet, WalletBmc, WalletForCreate};

    #[serial]
    #[tokio::test]
    async fn test_create_update_wallet() {
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

        let wallet_id = WalletBmc::create(
            &ctx,
            &mm,
            WalletForCreate {
                id: user_id,
                pub_key: "pubkey1".to_string(),
                sign_type: "eed259".to_string(),
                phrase: "ab cd".to_string(),
                last_faucet: None,
            },
        )
        .await
        .unwrap();

        let wallet = WalletBmc::get::<Wallet>(&ctx, &mm, wallet_id)
            .await
            .unwrap();

        assert_eq!(wallet.last_faucet, None);

        WalletBmc::update(
            &ctx,
            &mm,
            wallet_id,
            WalletForUpdateFaucet {
                last_faucet: 100000000000,
            },
        )
        .await
        .unwrap();

        let wallet = WalletBmc::get::<Wallet>(&ctx, &mm, wallet_id)
            .await
            .unwrap();

        assert_eq!(wallet.last_faucet, Some(100000000000));

        WalletBmc::delete(&ctx, &mm, wallet_id).await.unwrap();

        UserBmc::delete(&ctx, &mm, user_id).await.unwrap();
    }
}
// endregion:    --- Tests
