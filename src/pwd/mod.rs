// region:    --- Modules

mod error;

pub use self::error::{Error, Result};

use crate::utils::b64::b64u_encode;
use crate::{config, get_config};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use tracing::info;
use uuid::Uuid;
// endregion: --- Modules

// region:    --- Types

pub struct ContentToHash {
    pub content: String, // Clear content.
    pub salt: String,    // Clear salt.
}

// endregion: --- Types

// region:    --- Public Functions

/// Hash the password with the default scheme.
pub fn hash_pwd(to_hash: &ContentToHash) -> Result<String> {
    let key = &get_config().SERVICE_PWD_KEY;

    let hashed = hmac_sha512_hash(key, to_hash)?;

    Ok(format!("#01#{hashed}"))
}

/// Validate if an ContentToHash matches.
pub fn validate_pwd(enc_content: &ContentToHash, pwd_ref: &str) -> Result<()> {
    let pwd: String = hash_pwd(enc_content)?;

    if pwd.as_bytes().eq(pwd_ref.as_bytes()) {
        Ok(())
    } else {
        Err(Error::NotMatching)
    }
}
// endregion: --- Public Functions

// region: --- Helper Functions
fn hmac_sha512_hash(key: &[u8], to_hash: &ContentToHash) -> Result<String> {
    let ContentToHash { content, salt } = to_hash;

    // -- Create a HMAC-SHA-512 from key.
    let mut hmac_sha512 = Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::KeyFail)?;

    // -- Add content.
    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());

    // -- Finalize and b64u encode.
    let hmac_result = hmac_sha512.finalize();

    let result = b64u_encode(hmac_result.into_bytes());

    Ok(result)
}
// endregion: --- Helper Functions
