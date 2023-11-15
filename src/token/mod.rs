pub use self::error::{Error, Result};
use crate::models::User;
use crate::utils::b64::{b64u_decode_to_string, b64u_encode};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use std::fmt::Display;
use std::str::FromStr;
use uuid::Uuid;

use crate::get_config;
use crate::utils::time::{now_utc, now_utc_plus_sec_str, parse_utc};

mod error;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Token {
    pub ident: String,
    pub exp: String,
    pub sign: String,
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(token_str: &str) -> std::result::Result<Self, Self::Err> {
        let splits: Vec<&str> = token_str.split('.').collect();
        if splits.len() != 3 {
            return Err(Error::InvalidTokenFormat);
        }
        let (ident_b64u, exp_b64u, sign_b64u) = (splits[0], splits[1], splits[2]);

        Ok(Self {
            ident: b64u_decode_to_string(ident_b64u).map_err(|_| Error::CannotDecodeIdent)?,

            exp: b64u_decode_to_string(exp_b64u).map_err(|_| Error::CannotDecodeExp)?,

            sign: sign_b64u.to_string(),
        })
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            b64u_encode(&self.ident),
            b64u_encode(&self.exp),
            self.sign
        )
    }
}

pub fn create_token(user_itentity: &str, token_salt: Uuid) -> Result<Token> {
    let config = get_config();
    _generate_token(
        &user_itentity,
        config.SERVICE_TOKEN_DURATION_SEC,
        token_salt,
        &config.SERVICE_TOKEN_KEY,
    )
}

pub fn validate_token(token: Token, token_salt: Uuid) -> Result<()> {
    let config = get_config();
    _validate_token(token, token_salt, &config.SERVICE_TOKEN_KEY);
    Ok(())
}

fn _generate_token(
    ident: &str,
    duration: f64,
    token_salt: Uuid,
    secret_key: &[u8],
) -> Result<Token> {
    let exp = now_utc_plus_sec_str(duration);

    let signature = _sign(ident, &exp, token_salt, secret_key)?;

    Ok(Token {
        ident: ident.to_string(),
        exp,
        sign: signature,
    })
}

fn _validate_token(token: Token, token_salt: Uuid, secret_key: &[u8]) -> Result<()> {
    // validate signature
    let signature = _sign(&token.ident, &token.exp, token_salt, secret_key)?;
    if signature != token.sign {
        return Err(Error::SignatureNotMatching);
    }

    // validate expired time
    let expired_time = parse_utc(&token.exp).map_err(|_| Error::ExpNotIso)?;
    if expired_time < now_utc() {
        return Err(Error::Expired);
    }

    Ok(())
}

fn _sign(ident: &str, exp: &str, token_salt: Uuid, secret_key: &[u8]) -> Result<String> {
    let token_str = format!(
        "{}.{}.{}",
        b64u_encode(ident),
        b64u_encode(exp),
        token_salt.to_string()
    );

    // -- Create a HMAC-SHA-512 from key.
    let mut hmac_sha512 =
        Hmac::<Sha512>::new_from_slice(secret_key).map_err(|_| Error::HmacFailNewFromSlice)?;

    hmac_sha512.update(token_str.as_bytes());
    let result = b64u_encode(hmac_sha512.finalize().into_bytes());

    Ok(result)
}
