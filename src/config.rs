use crate::{utils::b64::b64u_decode, Error, Result};
use std::{env, str::FromStr, sync::OnceLock};

// public function to get the singleton config
pub fn get_config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();

    CONFIG.get_or_init(|| {
        Config::load_from_env().unwrap_or_else(|e| panic!("Error while loading config: {e:?}"))
    })
}

// region: --- Config and inplementation
#[allow(non_snake_case)]
pub struct Config {
    pub ENVIRONMENT: String,

    pub SERVICE_WEB_FOLDER: String,

    pub SERVICE_DB_URL: String,

    pub SERVICE_PWD_KEY: Vec<u8>,

    pub SERVICE_TOKEN_KEY: Vec<u8>,

    pub SERVICE_TOKEN_DURATION_SEC: f64,

    pub SERVICE_PASSWORD_SALT: String,

    pub DISCORD_CLIENT_ID: String,

    pub DISCORD_CLIENT_SECRET: String,

    pub DISCORD_TOKEN: String,

    pub CLOUDFLARE_SERVER_URL: String,

    pub APPLICATION_ID: u64,

    pub PACKAGE: String,

    pub SUI_CLIENT_ADDRESS: String,

    pub GAME_INFO_ID: String,

    pub GAME_ADMIN_ID: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            ENVIRONMENT: get_from_env("ENVIRONMENT")?,
            SERVICE_WEB_FOLDER: get_from_env("SERVICE_WEB_FOLDER")?,
            SERVICE_DB_URL: get_from_env("SERVICE_DB_URL")?,
            SERVICE_PWD_KEY: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,
            SERVICE_TOKEN_DURATION_SEC: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,
            SERVICE_TOKEN_KEY: get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")?,
            SERVICE_PASSWORD_SALT: get_from_env("SERVICE_PASSWORD_SALT")?,
            DISCORD_CLIENT_ID: get_from_env("DISCORD_CLIENT_ID")?,
            CLOUDFLARE_SERVER_URL: get_from_env("CLOUDFLARE_SERVER_URL")?,
            DISCORD_CLIENT_SECRET: get_from_env("DISCORD_CLIENT_SECRET")?,
            APPLICATION_ID: get_env_parse("APPLICATION_ID")?,
            PACKAGE: get_env_parse("PACKAGE")?,
            DISCORD_TOKEN: get_env_parse("DISCORD_TOKEN")?,
            SUI_CLIENT_ADDRESS: get_env_parse("SUI_CLIENT_ADDRESS")?,
            GAME_INFO_ID: get_env_parse("GAME_INFO_ID")?,
            GAME_ADMIN_ID: get_env_parse("GAME_ADMIN_ID")?,
        })
    }
}
// endregion: --- Config struct and inplementation

// region: --- Helper function
fn get_from_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissing(name))
}

fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = get_from_env(name)?;
    val.parse::<T>().map_err(|_| Error::WrongFormat(name))
}

fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
    b64u_decode(&get_from_env(name)?).map_err(|_| Error::WrongFormat(name))
}
// endregion: --- Helper function
