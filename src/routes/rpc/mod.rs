pub mod handler;
mod params;
mod user;

use crate::routes::error::Result;
use crate::routes::Error;
use crate::{ctx::Ctx, models::ModelManager};
use axum::response::IntoResponse;
use axum::{extract::State, response::Response, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::debug;

use self::handler::rpc_hanler;
pub use self::handler::RpcInfo;
pub use self::user::*;

pub fn routes(mm: ModelManager) -> Router {
    Router::new().route("/rpc", post(rpc_hanler)).with_state(mm)
}
