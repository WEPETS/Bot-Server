// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{str::FromStr, time::Duration};

use anyhow::bail;
use sui_json_rpc_types::{Coin, SuiObjectDataOptions};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_sdk::{
    sui_client_config::{SuiClientConfig, SuiEnv},
    wallet_context::WalletContext,
};
use tracing::info;

use reqwest::Client;
use serde_json::json;
use sui_sdk::types::{
    base_types::{ObjectID, SuiAddress},
    crypto::SignatureScheme::ED25519,
    digests::TransactionDigest,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    quorum_driver_types::ExecuteTransactionRequestType,
    transaction::{Argument, Command, Transaction, TransactionData},
};

use sui_sdk::{rpc_types::SuiTransactionBlockResponseOptions, SuiClient, SuiClientBuilder};

#[derive(serde::Deserialize)]
struct FaucetResponse {
    task: String,
    error: Option<String>,
}

// const SUI_FAUCET: &str = "https://faucet.devnet.sui.io/gas"; // devnet faucet

pub const SUI_FAUCET: &str = "https://faucet.testnet.sui.io/v1/gas"; // testnet faucet

/// Return a sui client to interact with the APIs and an active address from the local wallet.
///
/// This function sets up a wallet in case there is no wallet locally,
/// and ensures that the active address of the wallet has SUI on it.
/// If there is no SUI owned by the active address, then it will request
/// SUI from the faucet.
pub async fn setup_for_read() -> Result<(SuiClient, SuiAddress), anyhow::Error> {
    let client = SuiClientBuilder::default().build_devnet().await?;
    println!("Sui testnet version is: {}", client.api_version());
    let active_address =
        SuiAddress::from_str("0xb6c599cba8061a60acc445217823251cc1f0c8b4259a4ec4c8f51be9a8e361aa")?;

    println!("Wallet active address is: {active_address}");
    Ok((client, active_address))
}

/// Request tokens from the Faucet for the given address
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
    let resp = client
        .post(SUI_FAUCET)
        .header("Content-Type", "application/json")
        .json(&json_body)
        .send()
        .await?;
    println!(
        "Faucet request for address {address_str} has status: {}",
        resp.status()
    );
    println!("Waiting for the faucet to complete the gas request...");
    let faucet_resp: FaucetResponse = resp.json().await?;

    let task_id = if let Some(err) = faucet_resp.error {
        bail!("Faucet request was unsuccessful. Error is {err:?}")
    } else {
        faucet_resp.task
    };

    println!("Faucet request task id: {task_id}");

    let json_body = json![{
        "GetBatchSendStatusRequest": {
            "task_id": &task_id
        }
    }];

    let mut coin_id = "".to_string();

    // wait for the faucet to finsh the batch of token requests
    loop {
        let resp = client
            .get("https://faucet.testnet.sui.io/v1/status")
            .header("Content-Type", "application/json")
            .json(&json_body)
            .send()
            .await?;
        let text = resp.text().await?;
        if text.contains("SUCCEEDED") {
            let resp_json: serde_json::Value = serde_json::from_str(&text).unwrap();

            coin_id = <&str>::clone(
                &resp_json
                    .pointer("/status/transferred_gas_objects/sent/0/id")
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )
            .to_string();

            break;
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    // wait until the fullnode has the coin object, and check if it has the same owner
    loop {
        let owner = sui_client
            .read_api()
            .get_object_with_options(
                ObjectID::from_str(&coin_id)?,
                SuiObjectDataOptions::new().with_owner(),
            )
            .await?;

        if owner.owner().is_some() {
            let owner_address = owner.owner().unwrap().get_owner_address()?;
            if owner_address == address {
                break;
            }
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    Ok(())
}

fn main() {}
