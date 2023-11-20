pub mod error;

use self::error::Result;
use crate::{
    ctx::Ctx,
    models::{
        discord_profile::{self, DiscordProfile, DiscordProfileBmc},
        user::UserInfo,
        ModelManager, UserBmc,
    },
    utils::truncate_hex_string,
};
use futures::{future, StreamExt};
use serde::Deserialize;
use serenity::{futures, model::id::UserId};
use std::{env, str::FromStr};
use sui_json_rpc_types::Coin;
use sui_sdk::{types::base_types::SuiAddress, SuiClient};

#[derive(Debug, Deserialize)]
pub struct UserGameState {
    id: UserId,
    username: String,
    avatar: String,
    address: String,
    sui_coin: Option<Coin>,
    game_token: Option<Coin>,
}

impl UserGameState {
    fn new(
        id: i64,
        username: String,
        avatar: String,
        address: SuiAddress,
        sui_coin: Option<Coin>,
        game_token: Option<Coin>,
    ) -> Self {
        UserGameState {
            id: UserId(id as u64),
            username,
            avatar,
            address: address.to_string(),
            sui_coin,
            game_token,
        }
    }

    pub async fn new_state(
        sui_client: &SuiClient,
        mm: &ModelManager,
        user_info: &UserInfo,
    ) -> Result<Self> {
        let ctx = Ctx::root_ctx();
        let discord_profile = &user_info.discord;
        let wallet = &user_info.wallet;

        let address = SuiAddress::from_str(wallet.pub_key.as_str()).ok().unwrap();

        let sui_coins_stream = sui_client.coin_read_api().get_coins_stream(address, None);

        let sui_coin = sui_coins_stream
            // .skip_while(|c| future::ready(c.balance < 5_000_000))
            .boxed()
            .next()
            .await;

        Ok(UserGameState::new(
            discord_profile.discord_id,
            discord_profile.global_name.clone(),
            discord_profile.avatar.clone(),
            address,
            sui_coin,
            None,
        ))
    }

    pub fn get_game_state_board(&self) -> String {
        let mut game_state_board = String::from("----------------------------------------------\n");

        game_state_board.push_str(&format!("id: {:<30}\n", self.id));
        game_state_board.push_str(&format!("player: {:<30}\n", self.username));
        game_state_board.push_str(&format!(
            "address: {:<30}\n",
            truncate_hex_string(self.address.to_string().as_str(), 13),
        ));

        if let Some(sui_coin) = &self.sui_coin {
            game_state_board.push_str(&format!(
                "balance: {:<20}{:<10}\n",
                sui_coin.balance, sui_coin.coin_type,
            ));
        }

        if let Some(game_token) = &self.game_token {
            game_state_board.push_str(&format!(
                "balance: {:<20}{:<10}\n",
                game_token.balance, game_token.coin_type,
            ));
        }

        game_state_board
    }
}
