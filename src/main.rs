#![allow(unused)]

// region: -- Modules

// modules
mod _dev_init;
mod config;
mod ctx;
mod error;
mod log;
mod middlewares;
mod models;
mod pwd;
mod routes;
mod store;
mod token;
mod utils; // dev-test

// re-exports
pub use self::error::{Error, Result};
pub use config::get_config;

// imports
use crate::middlewares::{mw_ctx_resolve::mw_ctx_resolve, mw_reponse_map::mw_reponse_map};
use crate::routes::routes_static;
use axum::{middleware, Router};
use models::ModelManager;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::EnvFilter;

// endregion: -- Modules

#[tokio::main]
async fn main() -> Result<()> {
    // tracing debug
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // dev only
    if std::env::var("ENVIRONMENT").expect("ENV NOT FOUND") == "development" {
        _dev_init::init_db().await;
    }

    // model - store layer (DB)
    let mm = ModelManager::new().await?;

    // route defination
    let routes = Router::new()
        .merge(routes::routes(mm.clone()))
        .layer(middleware::map_response(mw_reponse_map))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .fallback_service(routes_static::serve_dir());

    // run the server in local
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    info!("--> {:<12} -- {addr}\n", "LISTENNING");

    axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await
        .unwrap();

    Ok(())
}
