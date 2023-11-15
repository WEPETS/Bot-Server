// region:    --- Imports
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
// endregion: --- Imports

// region:    --- Ctx Extractor Result/Error
pub type CtxExtResult = core::result::Result<Ctx, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
    TokenNotProvided,
    TokenWrongFormat,

    UserNotFound,
    ModelAccessError(String),
    FailValidate,
    CannotSetTokenCookie,

    CtxNotInRequestExt,
    CtxCreateFail(String),
}
// endregion: --- Ctx Extractor Result/Error
