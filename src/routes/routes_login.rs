// region: --- Imports
use axum::{
    extract::State,
    http::HeaderValue,
    response::{IntoResponse, Response},
    routing::{post, Route},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::{
    ctx::Ctx,
    get_config,
    models::{ModelManager, UserForAuth, UserForCreate, UserForLogin},
    pwd::{self, ContentToHash},
    token::{create_token, Token},
};
use crate::{
    models::{User, UserBmc},
    routes::{Error, Result},
};
// endregion: --- Imports

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/signup", post(signup_handler))
        // .route("/logoff", post(logoff_handler))
        .with_state(mm)
}

// region:    --- Login
pub async fn login_handler(
    State(mm): State<ModelManager>,
    Json(payload): Json<LoginPayload>,
) -> Result<Response> {
    debug!("{:<12} - login_handler", "HANDLER");

    let route_ctx = Ctx::root_ctx();

    let (res, token) = _login_handler(&route_ctx, &mm, payload).await?;
    let mut res = res.into_response();

    let token: String = token.to_string();

    res.headers_mut()
        .insert("authentication", HeaderValue::from_str(&token)?);

    Ok(res)
}

async fn _login_handler(
    ctx: &Ctx,
    mm: &ModelManager,
    payload: LoginPayload,
) -> Result<(Json<Value>, Token)> {
    let LoginPayload { username, password } = payload;
    // user exist
    let user = UserBmc::get_first_by_username::<UserForLogin>(ctx, mm, username.as_str())
        .await?
        .ok_or(Error::LoginFailUsernameNotFound)?;

    // check password
    let config = get_config();
    pwd::validate_pwd(
        &ContentToHash {
            content: password.clone(),
            salt: config.SERVICE_PASSWORD_SALT.to_string(),
        },
        &user.pwd,
    )
    .map_err(|_| Error::LoginFailPwdNotMatching { user_id: user.id })?;

    // create token
    let token = create_token(&user.username, user.token_salt)?;

    Ok((
        Json(json!({
            "result": {
                "success": true
            }
        })),
        token,
    ))
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}
// endregion: --- Login

// region:    --- Signup
async fn signup_handler(
    State(mm): State<ModelManager>,
    Json(payload): Json<SignupPayload>,
) -> Result<Response> {
    debug!("{:<12} - signup_handler", "HANDLER");
    let root_ctx = Ctx::root_ctx();
    let (res, token) = _signup_handler(&root_ctx, &mm, payload).await?;
    let mut res = res.into_response();

    let token: String = token.to_string();

    res.headers_mut()
        .insert("authentication", HeaderValue::from_str(&token)?);

    Ok(res)
}

async fn _signup_handler(
    ctx: &Ctx,
    mm: &ModelManager,
    payload: SignupPayload,
) -> Result<(Json<Value>, Token)> {
    // upwrap payload
    let SignupPayload { username, password } = payload;

    // check user exist
    UserBmc::get_first_by_username::<User>(&ctx, &mm, &username)
        .await?
        .map_or_else(
            || Ok(()),
            |_| Err(Error::SignUpFailedUserAlreadyExist(username.clone())),
        )?;

    // create user
    let data = UserForCreate {
        username,
        pwd: password,
    };
    let user_id = UserBmc::create(ctx, mm, data).await?;

    // create token
    let user = UserBmc::get::<UserForAuth>(&ctx, &mm, user_id).await?;
    let token = create_token(&user.username, user.token_salt)?;

    // response
    let res = Json(json!({
        "result": {
            "success": true,
            "user_id": user_id
        }
    }));

    Ok((res, token))
}

#[derive(Debug, Deserialize)]
pub struct SignupPayload {
    username: String,
    password: String,
}
// endregion: --- Signup

// region:    --- Logoff
// Logoff handler just doing nothing for now
// TODO: fix the logoff hanlder for reset token e.g.
pub async fn logoff_handler(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(payload): Json<LogoffPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - logoff_handler", "HANDLER");

    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
pub struct LogoffPayload {}
// endregion: --- Logoff
