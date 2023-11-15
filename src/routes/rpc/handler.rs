use crate::routes::error::Result;
use crate::routes::rpc::{delete_user, get_user, list_users, params, update_user};
use crate::routes::Error;
use crate::{ctx::Ctx, models::ModelManager};
use axum::response::IntoResponse;
use axum::{extract::State, response::Response, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{from_value, json, to_value, Value};
use tracing::debug;

// region:    --- RPC Types
#[derive(Deserialize)]
pub struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug)]
pub struct RpcInfo {
    pub id: Option<Value>,
    pub method: String,
}
// endregion: --- RPC Types

pub async fn rpc_hanler(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(rpc_request): Json<RpcRequest>,
) -> Response {
    let rpc_info = RpcInfo {
        id: rpc_request.id.clone(),
        method: rpc_request.method.clone(),
    };

    let mut res = _rpc_handler(mm, ctx, rpc_request).await.into_response();
    res.extensions_mut().insert(rpc_info);

    res
}

macro_rules! exec_rpc_fn {
    // With Params
    ($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_params:expr) => {{
        let rpc_fn_name = stringify!($rpc_fn);
        let params = $rpc_params.ok_or(Error::RpcMissingParams {
            rpc_method: rpc_fn_name.to_string(),
        })?;
        let params = from_value(params).map_err(|_| Error::RpcFailJsonParams {
            rpc_method: rpc_fn_name.to_string(),
        })?;
        $rpc_fn($ctx, $mm, params).await.map(to_value)??
    }};

    // Without Params
    ($rpc_fn:expr, $ctx:expr, $mm:expr) => {
        $rpc_fn($ctx, $mm).await.map(to_value)??
    };
}

pub async fn _rpc_handler(
    mm: ModelManager,
    ctx: Ctx,
    rpc_request: RpcRequest,
) -> Result<Json<Value>> {
    let RpcRequest {
        id: rpc_id,
        method: rpc_method,
        params: rpc_params,
    } = rpc_request;

    debug!("{:<12} - _rpc_handler - method: {rpc_method}", "HANDLER");

    let result_json: Value = match rpc_method.as_str() {
        // -- Task RPC methods.
        // "create_task" => exec_rpc_fn!(create_task, ctx, mm, rpc_params),
        "list_users" => exec_rpc_fn!(list_users, ctx, mm),
        "update_user" => exec_rpc_fn!(update_user, ctx, mm, rpc_params),
        "get_user" => exec_rpc_fn!(get_user, ctx, mm, rpc_params),
        "delete_user" => exec_rpc_fn!(delete_user, ctx, mm, rpc_params),

        // -- Fallback as Err.
        _ => return Err(Error::RpcMethodUnknown(rpc_method)),
    };

    let body_response = json!({
        "id": rpc_id,
        "result": result_json
    });

    Ok(Json(body_response))
}
