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
    pub SERVICE_WEB_FOLDER: String,

    pub SERVICE_DB_URL: String,

    pub SERVICE_PWD_KEY: Vec<u8>,

    pub SERVICE_TOKEN_KEY: Vec<u8>,

    pub SERVICE_TOKEN_DURATION_SEC: f64,

    pub SERVICE_PASSWORD_SALT: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            SERVICE_WEB_FOLDER: get_from_env("SERVICE_WEB_FOLDER")?,
            SERVICE_DB_URL: get_from_env("SERVICE_DB_URL")?,
            SERVICE_PWD_KEY: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,
            SERVICE_TOKEN_DURATION_SEC: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,
            SERVICE_TOKEN_KEY: get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")?,
            SERVICE_PASSWORD_SALT: get_from_env("SERVICE_PASSWORD_SALT")?,
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
