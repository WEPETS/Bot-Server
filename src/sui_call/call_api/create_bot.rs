use std::str::FromStr;

use crate::{get_config, sui_call::MODULE_NAME};

use super::error::{Error, Result};
use serde_json::{json, Value};
use shared_crypto::intent::Intent;
use sui_json_rpc_types::{
    ObjectChange, SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions,
};
use sui_keys::keystore::Keystore;
use sui_sdk::{json::SuiJsonValue, SuiClient};
use sui_types::{
    base_types::{ObjectID, SuiAddress},
    transaction::TransactionData,
};

use sui_keys::keystore::AccountKeystore;
use sui_types::{quorum_driver_types::ExecuteTransactionRequestType, transaction::Transaction};
use tracing::debug;
pub async fn create_bot(
    sui_client: &SuiClient,
    package_object_id: &ObjectID,
    player: SuiAddress,
    hp: u8,
    strength: u8,
    keystore: &Keystore,
) -> Result<ObjectID> {
    let config = get_config();

    let signer = SuiAddress::from_str(&config.SUI_CLIENT_ADDRESS).unwrap();

    // FIXME:  MODULE_NAME, FUNCTION_NAME

    let transaction_data = sui_client
        .transaction_builder()
        .move_call(
            signer,
            package_object_id.clone(),
            MODULE_NAME,
            "send_bot",
            vec![],
            vec![
                SuiJsonValue::from_str(config.GAME_INFO_ID.as_str()).unwrap(),
                SuiJsonValue::from_str(config.GAME_ADMIN_ID.as_str()).unwrap(),
                SuiJsonValue::from_str(player.to_string().as_str()).unwrap(),
                // SuiJsonValue::from_str(format!("{hp}").as_str()).unwrap(),
                // SuiJsonValue::from_str(format!("{strength}").as_str()).unwrap(),
                // SuiJsonValue::new(Value::Number(hp.into())).unwrap(),
                // SuiJsonValue::new(Value::Number(strength.into())).unwrap(),
                SuiJsonValue::from_str(&hp.to_string()).unwrap(),
                SuiJsonValue::from_str(&strength.to_string()).unwrap(),
            ],
            None,
            300000000,
        )
        .await
        .map_err(|e| {
            println!("{e:?}");
            Error::TransactionFail
        })?;
    // .map_err(|e| Error::TransactionFail)?;

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

    let obj_id = get_object_id(&response).unwrap();
    println!("Object ID: {:?}", obj_id);

    Ok(obj_id)
}

pub fn get_object_id(res: &SuiTransactionBlockResponse) -> Option<ObjectID> {
    res.object_changes.as_ref().and_then(|changes| {
        changes
            .iter()
            .filter_map(|change| match change {
                ObjectChange::Created { object_id, .. } => Some(object_id.clone()),
                _ => None,
            })
            .next()
    })
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

    use super::create_bot;

    #[serial]
    #[tokio::test]
    async fn test_create_bot_success() {
        dotenv().ok();

        let config = get_config();

        let keystore_path = Path::new("/home/ganzzi/.sui/sui_config/sui.keystore");
        let mut keystore =
            Keystore::from(FileBasedKeystore::new(&keystore_path.to_path_buf()).unwrap());

        let sui_client = SuiClientBuilder::default().build_devnet().await.unwrap();
        let package_id = ObjectID::from_str(&config.PACKAGE).unwrap();
        let player = SuiAddress::from_str(
            "0xb6c599cba8061a60acc445217823251cc1f0c8b4259a4ec4c8f51be9a8e361aa",
        )
        .unwrap();

        let a = create_bot(&sui_client, &package_id, player, 50, 5, &keystore)
            .await
            .map_err(|e| println!("error: {e:?}"));

        println!("{a:?}");
    }
}
// endregion:    --- Tests
