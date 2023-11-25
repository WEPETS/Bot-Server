use anyhow;
use dotenvy::dotenv;
use serenity::async_trait;
use serenity::builder::{CreateApplicationCommand, CreateApplicationCommands};
use serenity::futures::StreamExt;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::{ApplicationId, ChannelId, GuildId, UserId};
use serenity::prelude::*;
use std::env;
use std::str::FromStr;
use sui_keys::keystore::Keystore;
use sui_sdk::types::base_types::{ObjectID, SuiAddress};
use sui_sdk::{SuiClient, SuiClientBuilder};
use tracing::{debug, info};

use crate::commands;
use crate::config::Config;
use crate::ctx::Ctx;
use crate::game_state::UserGameState;
use crate::models::discord_profile::{DiscordProfile, DiscordProfileBmc};
use crate::models::user::UserInfo;
use crate::models::{ModelManager, UserBmc};

pub struct Handler {
    pub sui_client: SuiClient,
    pub package_id: ObjectID,
    pub default_address: SuiAddress,
    pub mm: ModelManager,
    pub config: &'static Config,
    pub keystore: Keystore,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_str() {
            "hello" => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "world").await {
                    println!("Error sending message: {:?}", why);
                }
            }
            _ => (),
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let mut res = "".to_string();

            // TODO: add command -> function
            if (is_player(&self.mm, i64::from(command.user.id)).await) {
                let user_info = UserBmc::get_user_info_by_discord_id(
                    &Ctx::root_ctx(),
                    &self.mm,
                    command.user.id.into(),
                )
                .await
                .expect("something wrong when get user info");

                match command.data.name.as_str() {
                    "state" => {
                        res = get_game_state(&self, &user_info).await;
                    }
                    // "hunt" => {
                    //     res = do_hunt(&self, &command.data.options, &user_info).await;
                    // }
                    "battle" => {
                        if check_sui(&self.sui_client, &user_info.wallet.pub_key).await {
                            res = do_battle(&self, &command.data.options, &user_info).await;
                        } else {
                            res = "you have no SUI coin".into();
                        }
                    }
                    _ => res = "Player already exist".to_string(),
                };
            } else {
                match command.data.name.as_str() {
                    "register" => {
                        let user_id = command.user.id; // 530364905812131840
                        let r_uri = self
                            .config
                            .CLOUDFLARE_SERVER_URL
                            .clone()
                            .strip_prefix("https://")
                            .unwrap_or_default()
                            .to_string();

                        res = format!("Enter this link to authorize and register: https://discord.com/api/oauth2/authorize?client_id=1172504182691991562&redirect_uri=https%3A%2F%2F{}%2Fauth%2Fregister&response_type=code&scope=identify", r_uri);
                        // res = "https://discord.com/api/oauth2/authorize?client_id=1172504182691991562&redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fauth%2Fregister&response_type=code&scope=identify".to_string();
                    }
                    _ => res = "Not a player".to_string(),
                };
            }

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(res))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let mut cmds2 = ApplicationId(self.config.APPLICATION_ID);
        let cs = Command::set_global_application_commands(ctx.http.clone(), |commands| {
            // commands.create_application_command(|command| commands::hunt::register(command));
            commands.create_application_command(|command| commands::battle::register(command));
            commands.create_application_command(|command| {
                command.name("register").description("register to play...")
            });
            commands.create_application_command(|command| {
                command.name("state").description("Get your game state.")
            })
        })
        .await;

        // info!("command: {cs:?}");
    }
}

async fn is_player(mm: &ModelManager, discord_id: i64) -> bool {
    let ctx = &Ctx::root_ctx();
    DiscordProfileBmc::get_by_discord_id::<DiscordProfile>(ctx, mm, discord_id)
        .await
        .is_ok()
}

async fn get_game_state(handler: &Handler, user_info: &UserInfo) -> String {
    let a = UserGameState::new_state(
        &handler.sui_client,
        &handler.mm,
        &handler.package_id,
        user_info,
    )
    .await
    .map_err(|e| debug!("error: {e:?}"))
    .unwrap()
    .get_game_state_board();
    a
}

// async fn do_hunt(handler: &Handler, options: &[CommandDataOption], user_info: &UserInfo) -> String {
//     let signer = get_signer(&user_info.wallet.pub_key);
//     let _data = commands::hunt::do_hunt(&handler.sui_client, &handler.package_id, options, signer)
//         .await
//         .map_err(|e| println!("error: {e:?}"))
//         .unwrap();

//     get_game_state(&handler, user_info).await
// }

async fn check_sui(sui_client: &SuiClient, address: &String) -> bool {
    let sui_coins_stream = sui_client.coin_read_api().get_coins_stream(
        SuiAddress::from_str(address.clone().as_str()).unwrap(),
        None,
    );

    let sui_coin = sui_coins_stream.boxed().next().await;

    sui_coin.is_some()
}
async fn do_battle(
    handler: &Handler,
    options: &[CommandDataOption],
    user_info: &UserInfo,
) -> String {
    let signer = get_signer(&user_info.wallet.pub_key);
    let _data = commands::battle::do_battle(
        &handler.sui_client,
        &handler.package_id,
        // &handler.keystore,
        options,
        signer,
    )
    .await
    .map_err(|e| println!("error: {e:?}"))
    .unwrap();

    get_game_state(&handler, user_info).await
}

fn get_signer(pub_key: &str) -> SuiAddress {
    SuiAddress::from_str(pub_key).unwrap_or_default()
}
