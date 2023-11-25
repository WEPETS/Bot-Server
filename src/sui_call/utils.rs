use std::{str::FromStr, time::Duration};

use reqwest::Client;
use serde_json::json;
use sui_json_rpc_types::SuiObjectDataOptions;
use sui_sdk::SuiClient;
use sui_types::base_types::{ObjectID, SuiAddress};
use tracing::debug;

// pub const SUI_FAUCET: &str = "https://faucet.testnet.sui.io/v1"; // testnet faucet

const SUI_FAUCET: &str = "https://faucet.devnet.sui.io"; // devnet faucet

// local faucet
// pub const SUI_FAUCET: &str = "http://0.0.0.0:9000";

#[derive(serde::Deserialize)]
struct FaucetResponse {
    task: String,
    error: Option<String>,
}

#[allow(unused_assignments)]
pub async fn request_tokens_from_faucet(
    address: SuiAddress,
    sui_client: &SuiClient,
) -> Result<(), anyhow::Error> {
    let address_str = address.to_string();
    let json_body = json![{
        "FixedAmountRequest": {
            "recipient": &address_str
        }
    }];

    // make the request to the faucet JSON RPC API for coin
    let client = Client::new();
    let mut url = SUI_FAUCET.to_string();
    url.push_str("/gas");

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&json_body)
        .send()
        .await?;
    println!(
        "Faucet request for address {address_str} has status: {}",
        resp.status()
    );
    println!("Waiting for the faucet to complete the gas request...");

    println!("{resp:?}");

    Ok(())
}

// region:    --- Tests
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use dotenvy::dotenv;
    use serial_test::serial;
    use sui_sdk::{SuiClient, SuiClientBuilder};
    use sui_types::base_types::{ObjectID, SuiAddress};
    use tracing::{debug, info};

    use crate::get_config;

    use super::request_tokens_from_faucet;

    #[serial]
    #[tokio::test]
    async fn test_faucet_success() {
        dotenv().ok();

        let sui_client = SuiClientBuilder::default().build_devnet().await.unwrap();

        let address = SuiAddress::from_str(
            "0xdb96399b7daeac4613a8494a30cf371206cff2ea4d19924d87ddf151d0d3a1c7",
        )
        .unwrap();

        let rs = request_tokens_from_faucet(address, &sui_client)
            .await
            .map_err(|e| println!("e is {e:?}"));

        println!("{rs:?}")
    }
}
// endregion:    --- Tests
