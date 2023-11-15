// region:    --- Imports
use crate::{
    ctx::Ctx,
    log::log_request,
    routes::{rpc::RpcInfo, Error, Result},
};
use axum::{
    http::{HeaderValue, Method, Request, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, to_value};
use tracing::debug;
use uuid::Uuid;
// endregion:    --- Imports

// Middleware for mapping & logging client response
pub async fn mw_reponse_map(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    debug!("{:<12} - mw_reponse_map", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    let rpc_info = res.extensions().get::<RpcInfo>();

    let route_error = res.extensions().get::<Error>();

    let code_n_error = route_error.map(|e| e.client_status_and_error());

    // convert status code & error into response
    let error_res = code_n_error.as_ref().map(|(status_code, error)| {
        let error = to_value(error).ok();
        let msg = error.as_ref().and_then(|v| v.get("message"));
        let detail = error.as_ref().and_then(|v| v.get("detail"));

        let error_body = json!({
            "id": rpc_info.as_ref().map(|rpc| rpc.id.clone()),
            "error": {
                "message": msg, // Variant name
                "data": {
                    "req_uuid": uuid.to_string(),
                    "detail": detail
                },
            }
        });

        (*status_code, Json(error_body)).into_response()
    });

    let client_error = code_n_error.unzip().1;
    let _ = log_request(
        uuid,
        req_method,
        uri,
        rpc_info,
        ctx,
        route_error,
        client_error,
    );

    error_res.unwrap_or(res)
}
