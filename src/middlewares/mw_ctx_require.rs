use axum::{http::Request, middleware::Next, response::Response};
use tracing::debug;

use crate::{ctx::Ctx, routes::Result};

// Middlewre for rpc routes
pub async fn mw_ctx_require<B>(
    ctx: Result<Ctx>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}
