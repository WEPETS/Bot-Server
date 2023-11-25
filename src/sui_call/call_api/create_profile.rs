use super::error::{Error, Result};
use crate::{get_config, sui_call::MODULE_NAME};
use shared_crypto::intent::Intent;
use sui_json_rpc_types::{SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions};

use serde_json::json;
use std::str::FromStr;
use sui_keys::keystore::{AccountKeystore, Keystore};
use sui_sdk::{json::SuiJsonValue, SuiClient};
use sui_types::{
    base_types::{ObjectID, SuiAddress},
    quorum_driver_types::ExecuteTransactionRequestType,
    transaction::{Transaction, TransactionData},
};
use tracing::debug;
pub async fn create_profile(
    sui_client: &SuiClient,
    package_object_id: &ObjectID,
    player: SuiAddress,
    keystore: &Keystore,
) -> Result<SuiTransactionBlockResponse> {
    let config = get_config();

    let signer = SuiAddress::from_str(&config.SUI_CLIENT_ADDRESS).unwrap();

    // FIXME:  MODULE_NAME, FUNCTION_NAME

    let transaction_data = sui_client
        .transaction_builder()
        .move_call(
            signer,
            package_object_id.clone(),
            MODULE_NAME,
            "create_profile",
            vec![],
            vec![
                SuiJsonValue::from_str(config.GAME_INFO_ID.as_str()).unwrap(),
                SuiJsonValue::from_str(config.GAME_ADMIN_ID.as_str()).unwrap(),
                SuiJsonValue::from_str(player.to_string().as_str()).unwrap(),
            ],
            None,
            300000000,
        )
        .await
        .map_err(|e| {
            println!("{e:?}");
            Error::TransactionFail
        })?;

    // Sign transaction.
    let signature = keystore
        .sign_secure(&signer, &transaction_data, Intent::sui_transaction())
        .map_err(|e| debug!("{e:?}"))
        .unwrap();

    // Execute the transaction.

    let response: SuiTransactionBlockResponse = sui_client
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(transaction_data, Intent::sui_transaction(), vec![signature]),
            SuiTransactionBlockResponseOptions::full_content(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await
        .map_err(|e| println!("{e:?}"))
        .unwrap();

    Ok(response)
}

// region:    --- Tests
#[cfg(test)]
mod tests {
    use std::{path::Path, str::FromStr};

    use dotenvy::dotenv;
    use serial_test::serial;
    use sui_keys::keystore::{FileBasedKeystore, Keystore};
    use sui_sdk::{SuiClient, SuiClientBuilder};
    use sui_types::base_types::{ObjectID, SuiAddress};
    use tracing::{debug, info};

    use crate::get_config;

    use super::create_profile;

    #[serial]
    #[tokio::test]
    async fn test_create_profile_success() {
        dotenv().ok();

        let config = get_config();

        let keystore_path = Path::new("/home/ganzzi/.sui/sui_config/sui.keystore");
        let mut keystore =
            Keystore::from(FileBasedKeystore::new(&keystore_path.to_path_buf()).unwrap());

        let sui_client = SuiClientBuilder::default().build_devnet().await.unwrap();
        let package_id = ObjectID::from_str(&config.PACKAGE).unwrap();
        let player = SuiAddress::from_str(
            "0xdb96399b7daeac4613a8494a30cf371206cff2ea4d19924d87ddf151d0d3a1c7",
        )
        .unwrap();

        let a = create_profile(&sui_client, &package_id, player, &keystore)
            .await
            .map_err(|e| println!("error: {e:?}"));

        println!("{a:?}")
    }
}
// endregion:    --- Tests
