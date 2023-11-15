// region: -- Modules

mod db;

use tokio::sync::OnceCell;
use tracing::info;

use crate::{ctx::Ctx, models::ModelManager};
// endregion: -- Modules

/// init db for dev
pub async fn init_db() {
    static INIT: OnceCell<()> = OnceCell::const_new();

    INIT.get_or_init(|| async {
        info!("{:<12} - init_dev_all()", "FOR-DEV-ONLY");

        db::init_db_for_dev().await.unwrap();
    })
    .await;
}

/// for testing only
pub async fn init_db_for_test() -> ModelManager {
    static INIT: OnceCell<ModelManager> = OnceCell::const_new();

    let mm = INIT
        .get_or_init(|| async {
            init_db().await;
            ModelManager::new().await.unwrap()
        })
        .await;

    mm.clone()
}
