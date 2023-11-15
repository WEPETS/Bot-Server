// region:    --- Imports
use super::error::{CtxExtError, CtxExtResult};
use crate::{
    ctx::Ctx,
    models::{ModelManager, UserBmc, UserForAuth},
    routes::{Error, Result},
    token::{validate_token, Token},
};
use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use serde::Serialize;
use tracing::debug;
// endregion:    --- Imports

// Middleware for all routes
pub async fn mw_ctx_resolve<B>(
    State(mm): State<ModelManager>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

    let ctx_rs = _ctx_resolve(&mm, &req).await;

    if ctx_rs.is_err() && !matches!(ctx_rs, Err(CtxExtError::TokenNotProvided)) {}

    // Add ctx_rs to req extension
    req.extensions_mut().insert(ctx_rs);

    Ok(next.run(req).await)
}

async fn _ctx_resolve<B>(mm: &ModelManager, req: &Request<B>) -> CtxExtResult {
    // -- get token from header
    let token = req
        .headers()
        .get("authentication")
        .map(|t| t.to_str())
        .ok_or(CtxExtError::TokenNotProvided)?
        .unwrap();

    // -- Parse Token
    let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

    // --  get user
    let root_ctx = Ctx::root_ctx();
    let user: UserForAuth = UserBmc::get_first_by_username(&root_ctx, &mm, token.ident.as_str())
        .await
        .map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
        .ok_or(CtxExtError::UserNotFound)?;

    // -- validate token
    validate_token(token, user.token_salt).map_err(|_| CtxExtError::FailValidate)?;

    Ctx::new(user.id).map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

// region:    --- Ctx Extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        debug!("{:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            .get::<CtxExtResult>()
            .ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .clone()
            .map_err(Error::CtxExt)
    }
}
// endregion: --- Ctx Extractor
