use std::path::Path;
use std::str::FromStr;

use serenity::builder;
use serenity::model::application::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::command::CommandOptionType;
use shared_crypto::intent::Intent;
use sui_json_rpc_types::{SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions};
use sui_keys::keystore::{self, AccountKeystore, FileBasedKeystore, Keystore};
use sui_sdk::json::SuiJsonValue;
use sui_sdk::SuiClient;
use sui_types::base_types::{ObjectID, SuiAddress};
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use sui_types::transaction::{Transaction, TransactionData};
use tracing::debug;

use crate::get_config;
use crate::sui_call::call_api::create_bot::get_object_id;
use crate::sui_call::read_api::owned_objects::WePetGame;
use crate::sui_call::sui_move_object::hero_obj::SuiHeroObject;
use crate::sui_call::{HERO_OBJECT_NAME, MODULE_NAME};

// TODO: implement function
pub async fn do_battle(
    sui_client: &SuiClient,
    package_object_id: &ObjectID,
    // keystore: &Keystore,
    options: &[CommandDataOption],
    signer: SuiAddress,
) -> Result<(), anyhow::Error> {
    let keystore_path = Path::new("/home/ganzzi/.sui/sui_config/sui.keystore");
    let mut keystore =
        Keystore::from(FileBasedKeystore::new(&keystore_path.to_path_buf()).unwrap());

    let wepet_game = WePetGame::new(
        sui_client.clone(),
        signer,
        package_object_id.to_string().as_str(),
    );
    let hero = wepet_game
        .get_sui_obj_first::<SuiHeroObject>(HERO_OBJECT_NAME)
        .await
        .ok()
        .unwrap();

    println!("hero data: \n{hero:?}\n");

    let option = get_option(options, 0).unwrap();
    let option2 = get_option(options, 1).unwrap();

    println!("option data: \n{option:?}\n");
    println!("option2 data: \n{option2:?}\n");

    // FIXME:  MODULE_NAME, FUNCTION_NAME
    if let (CommandDataOptionValue::String(pet), CommandDataOptionValue::String(bot)) =
        (option, option2)
    {
        let config = get_config();

        // FIXME:  MODULE_NAME, FUNCTION_NAME

        let transaction_data = sui_client
            .transaction_builder()
            .move_call(
                signer,
                package_object_id.clone(),
                MODULE_NAME,
                "huntbot",
                vec![],
                vec![
                    SuiJsonValue::from_str(config.GAME_INFO_ID.as_str()).unwrap(),
                    SuiJsonValue::from_str(hero.id.as_str()).unwrap(),
                    SuiJsonValue::from_str(pet.as_str()).unwrap(),
                    SuiJsonValue::from_str(bot.as_str()).unwrap(),
                ],
                None,
                300000000,
            )
            .await
            .map_err(|e| {
                println!("{e:?}");
                anyhow::Error::msg("sui transaction fail")
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
                Transaction::from_data(
                    transaction_data,
                    Intent::sui_transaction(),
                    vec![signature],
                ),
                SuiTransactionBlockResponseOptions::full_content(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await
            .map_err(|e| println!("{e:?}"))
            .unwrap();

        Ok(())
    } else {
        Err(anyhow::Error::msg("error message"))
    }
}

fn get_option(options: &[CommandDataOption], index: usize) -> Option<&CommandDataOptionValue> {
    let option = options
        .get(index)
        .expect("Expected arg option")
        .resolved
        .as_ref();
    option
}

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("battle")
        .description("Command for battle ...")
        .create_option(|option| {
            option
                .name("pet")
                .description("Your pet")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("bot")
                .description("choose bot to battle")
                .kind(CommandOptionType::String)
                .required(true)
        })
}
