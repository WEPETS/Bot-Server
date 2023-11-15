use axum::http::header::InvalidHeaderValue;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::debug;

use crate::middlewares::error::CtxExtError;
use crate::{middlewares, models, pwd, routes, token};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // -- RPC
    RpcMethodUnknown(String),
    RpcMissingParams { rpc_method: String },
    RpcFailJsonParams { rpc_method: String },
    RpcNoPermission,

    // -- Login
    LoginFailUsernameNotFound,
    LoginFailPwdNotMatching { user_id: i64 },
    SignUpFailedUserAlreadyExist(String),

    // -- CtxExtError
    CtxExt(CtxExtError),

    // -- Modules
    Model(models::Error),
    Pwd(pwd::Error),
    Token(token::Error),

    // -- External Modules
    SerdeJson(String),
}

// region:    --- Froms
impl From<models::Error> for Error {
    fn from(val: models::Error) -> Self {
        Error::Model(val)
    }
}

impl From<pwd::Error> for Error {
    fn from(val: pwd::Error) -> Self {
        Self::Pwd(val)
    }
}

impl From<token::Error> for Error {
    fn from(val: token::Error) -> Self {
        Self::Token(val)
    }
}

impl From<serde_json::Error> for Error {
    fn from(val: serde_json::Error) -> Self {
        Self::SerdeJson(val.to_string())
    }
}

impl From<InvalidHeaderValue> for Error {
    fn from(err: InvalidHeaderValue) -> Self {
        Error::SerdeJson(format!("Invalid header value: {}", err))
    }
}
// endregion: --- Froms

// region:    --- Axum IntoResponse
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        debug!("{:<12} - model::Error {self:?}", "INTO_RES");

        // Create a placeholder Axum reponse.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the reponse.
        response.extensions_mut().insert(self);

        response
    }
}
// endregion: --- Axum IntoResponse

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate

// region:    --- Client Error

/// From the root error to the http status code and ClientError
impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use routes::Error::*;

        #[allow(unreachable_patterns)]
        match self {
            // -- Login
            LoginFailUsernameNotFound | LoginFailPwdNotMatching { .. } => {
                (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL)
            }

            SignUpFailedUserAlreadyExist(username) => (
                StatusCode::CREATED,
                ClientError::SIGN_UP_FAIL(username.clone()),
            ),

            // -- Auth
            CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            // -- Model
            Model(models::Error::EntityNotFound { entity, id }) => (
                StatusCode::BAD_REQUEST,
                ClientError::ENTITY_NOT_FOUND { entity, id: *id },
            ),

            // -- Fallback.
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    ENTITY_NOT_FOUND { entity: &'static str, id: i64 },

    SIGN_UP_FAIL(String),

    SERVICE_ERROR,
}
// endregion: --- Client Error
