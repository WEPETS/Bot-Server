use std::path::Path;

// region: --- Imports
use axum::{
    extract::Query,
    extract::State,
    http::HeaderValue,
    response::{IntoResponse, Response},
    routing::{get, post, Route},
    Json, Router,
};
use serde::{de::Visitor, Deserialize, Deserializer};
use serde_json::{json, Value};
use tracing::{debug, info};

use sui_keys::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use sui_types::crypto::{DefaultHash, SignatureScheme, SuiSignatureInner};
use sui_types::{
    base_types::{SuiAddress, SUI_ADDRESS_LENGTH},
    crypto::Ed25519SuiSignature,
};

use crate::{
    ctx::Ctx,
    get_config,
    models::{
        discord_profile::{self, DiscordProfile, DiscordProfileBmc, DiscordProfileForCreate},
        wallet::{WalletBmc, WalletForCreate},
        ModelManager, UserForAuth, UserForCreate, UserForLogin,
    },
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
        .route("/register", get(register_hanlder))
        .with_state(mm)
}

// region:    --- Signup
async fn register_hanlder(
    query: Query<CodeQuery>,
    State(mm): State<ModelManager>,
) -> Result<Response> {
    debug!("{:<12} - register_handler", "HANDLER");
    let root_ctx = Ctx::root_ctx();

    // get the token
    let token = query.0.code;

    // get user discord info
    let user_info = get_user_info(token.as_str()).await?;

    // create new user
    let res = _register_handler(&root_ctx, &mm, user_info).await?;

    Ok(res.into_response())
}

async fn get_user_info(code: &str) -> Result<DiscordUserInfResonse> {
    let config = get_config();

    let reridect_uri = format!("{}/auth/register", config.CLOUDFLARE_SERVER_URL);
    let form_data = [
        ("code", code),
        ("client_id", config.DISCORD_CLIENT_ID.as_str()),
        ("client_secret", config.DISCORD_CLIENT_SECRET.as_str()),
        ("grant_type", "authorization_code"),
        ("redirect_uri", reridect_uri.as_str()),
    ];

    let client = reqwest::Client::new();
    let response = client
        .post("https://discord.com/api/oauth2/token")
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .form(&form_data)
        .send()
        .await
        .map_err(|_| Error::DiscordTokenRequestFail)?;

    let response_body = response
        .json::<DiscordTokenResponse>()
        .await
        .map_err(|_| Error::ParseTokenFail)?;

    let user_info_res = client
        .get("https://discord.com/api/users/@me")
        .header(
            "Authorization",
            format!("Bearer {}", response_body.access_token).as_str(),
        )
        .send()
        .await
        .map_err(|_| Error::DiscordTokenRequestFail)?;

    let user_info_json = user_info_res
        .json::<DiscordUserInfResonse>()
        .await
        .map_err(|_| Error::ParseTokenFail)?;

    Ok({ user_info_json })
}

async fn _register_handler(
    ctx: &Ctx,
    mm: &ModelManager,
    user_info: DiscordUserInfResonse,
) -> Result<Json<Value>> {
    // check user exist
    match DiscordProfileBmc::get_by_discord_id::<DiscordProfile>(ctx, mm, user_info.id).await {
        Ok(_) => {
            Error::SignUpFailedUserAlreadyExist(user_info.username.clone());
        }
        Err(e) => (),
    }

    // create user
    let user_c = UserForCreate {
        username: Some(user_info.username.clone()),
        pwd: None,
        email: None,
    };
    let user_id = UserBmc::create(ctx, mm, user_c).await?;

    // create discord profile
    let discord_profile_c = DiscordProfileForCreate {
        id: user_id,
        discord_id: user_info.id,
        username: user_info.username.clone(),
        global_name: user_info.global_name,
        avatar: user_info.avatar,
    };
    let d_id = DiscordProfileBmc::create(ctx, mm, discord_profile_c).await?;

    // create wallet
    let keystore_path = Path::new("/home/ganzzi/.sui/sui_config/sui.keystore");
    let mut keystore =
        Keystore::from(FileBasedKeystore::new(&keystore_path.to_path_buf()).unwrap());

    let (address, phrase, scheme) = keystore
        .generate_and_add_new_key(SignatureScheme::ED25519, None, None)
        .unwrap();

    let sign_type = match scheme {
        SignatureScheme::ED25519 => "ed25519",
        _ => "",
    }
    .to_string();

    let wallet_c = WalletForCreate {
        id: user_id,
        pub_key: address.to_string(),
        sign_type,
        phrase,
    };
    WalletBmc::create(ctx, mm, wallet_c).await?;

    // response
    let res = Json(json!({
        "result": {
            "success": true,
        }
    }));

    Ok((res))
}

#[derive(Debug, serde::Deserialize)]
struct CodeQuery {
    code: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupPayload {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordUserInfResonse {
    #[serde(deserialize_with = "parse")]
    pub id: i64,
    pub username: String,
    pub avatar: String,
    pub global_name: String,
    // other fiels..
    // pub discriminator: String,
    // pub public_flags: u64,
    // pub premium_type: u64,
    // pub flags: u64,
    // pub banner: Option<String>,
    // pub accent_color: Option<u64>,
    // pub avatar_decoration_data: Option<String>,
    // pub banner_color: Option<String>,
    // pub mfa_enabled: bool,
    // pub locale: String,
}

fn parse<'de, T, D>(de: D) -> core::result::Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    Ok(String::deserialize(de)?
        .parse()
        .map_err(serde::de::Error::custom)?)
}

#[derive(Debug, Deserialize)]
struct DiscordTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i32,
    refresh_token: String,
    scope: String,
}
// endregion: --- Signup
