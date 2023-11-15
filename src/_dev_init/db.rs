// region: --- IMPORTS
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tracing::info;

use crate::{ctx::Ctx, models::ModelManager};
// endregion: --- IMPORTS

// region: --- TYPES & CONSTANTS
// db pool for testing
type DBPool = Pool<Postgres>;
// db url
const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@localhost/postgres";
const PG_DEV_APP_URL: &str = "postgres://app_user:dev_pwd@localhost/app_db";
// sql file & dir name
const SQL_RECREATE_DB_FILE_NAME: &str = "00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_inits";
// demo password for update user
const DEMO_PWD: &str = "abc123";
// endregion: --- TYPES & CONSTANTS

// init db for dev only
pub(in crate::_dev_init) async fn init_db_for_dev() -> Result<(), Box<dyn std::error::Error>> {
    info!("{:<12} - init_db_for_dev()", "FOR-DEV-ONLY");

    let sql_dir: PathBuf = get_sql_dir_name();

    // -- Re-create the app_db/app_user with the postgres user.
    recreate_db_and_user(&sql_dir).await?;

    // execute files in sql/dev_inits folder
    exec_sql_files(&sql_dir).await?;

    // update user password for dev
    // update_user_for_dev().await?;

    Ok(())
}

async fn new_db_pool(db_url: &str) -> Result<DBPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect(db_url)
        .await
}

async fn recreate_db_and_user(sql_dir: &PathBuf) -> Result<(), sqlx::Error> {
    let sql_recreate_db_file = sql_dir.join(SQL_RECREATE_DB_FILE_NAME);
    let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
    exec_from_file(&root_db, &sql_recreate_db_file).await?;
    Ok(())
}

async fn exec_sql_files(sql_dir: &PathBuf) -> Result<(), sqlx::Error> {
    // -- Get sql files.
    let mut paths: Vec<PathBuf> = fs::read_dir(sql_dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    paths.sort();

    // -- SQL Execute each file.
    let app_db = new_db_pool(PG_DEV_APP_URL).await?;

    for path in paths {
        let path_str = path.to_string_lossy();

        if path_str.ends_with(".sql") && !path_str.ends_with(SQL_RECREATE_DB_FILE_NAME) {
            exec_from_file(&app_db, &path).await?;
        }
    }
    Ok(())
}

async fn exec_from_file(db_pool: &DBPool, file: &Path) -> Result<(), sqlx::Error> {
    info!("{:<12} - exec_from_file: {file:?}", "FOR-DEV-ONLY");

    // -- Read the file.
    let content = fs::read_to_string(file)?;

    // FIXME: Make the split more sql proof.
    let sqls: Vec<&str> = content.split(';').collect();

    for sql in sqls {
        sqlx::query(sql).execute(db_pool).await?;
    }

    Ok(())
}

fn get_sql_dir_name() -> PathBuf {
    let current_dir = std::env::current_dir().unwrap();
    let v: Vec<_> = current_dir.components().collect();
    let path_comp = v.get(v.len().wrapping_sub(3));
    let base_dir = if Some(true) == path_comp.map(|c| c.as_os_str() == "crates") {
        v[..v.len() - 3].iter().collect::<PathBuf>()
    } else {
        current_dir.clone()
    };
    base_dir.join(SQL_DIR)
}
