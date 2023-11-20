mod error;
pub mod routes_login;
pub mod routes_static;
pub mod rpc;

use axum::routing::post;
use axum::{middleware, Router};

use crate::middlewares::mw_ctx_require::mw_ctx_require;
use crate::models::ModelManager;

pub use self::error::ClientError;
pub use self::error::{Error, Result};
use self::rpc::handler::rpc_hanler;

pub fn routes(mm: ModelManager) -> Router {
    Router::new().nest("/auth", routes_login::routes(mm.clone()))
    // .nest(
    //     "/api",
    //     rpc::routes(mm.clone()).route_layer(middleware::from_fn(mw_ctx_require)),
    // )
}
