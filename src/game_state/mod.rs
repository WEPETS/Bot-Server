pub mod error;

use self::error::Result;
use crate::{
    ctx::Ctx,
    models::{
        discord_profile::{self, DiscordProfile, DiscordProfileBmc},
        user::UserInfo,
        ModelManager, UserBmc,
    },
    sui_call::{
        read_api::owned_objects::WePetGame,
        sui_move_object::{
            admin_obj::SuiAdminObject, bot_obj::SuiBotObject, hero_obj::SuiHeroObject,
            pet_obj::SuiPetObject,
        },
        ADMIN_OBJECT_NAME, BOT_OBJECT_NAME, HERO_OBJECT_NAME, PET_OBJECT_NAME,
    },
    utils::truncate_hex_string,
};
use futures::{future, StreamExt};
use serde::Deserialize;
use serenity::{futures, model::id::UserId};
use std::{env, str::FromStr};
use sui_json_rpc_types::Coin;
use sui_sdk::{types::base_types::SuiAddress, SuiClient};
use sui_types::base_types::ObjectID;

#[derive(Debug, Deserialize)]
pub struct UserGameState {
    id: UserId,
    username: String,
    avatar: String,
    address: String,
    sui_coin: Option<Coin>,
    game_token: Option<Coin>,
    admin: Option<SuiAdminObject>,
    hero: Option<SuiHeroObject>,
    pet: Option<SuiPetObject>,
    bot: Option<SuiBotObject>,
}

impl UserGameState {
    fn new(
        id: i64,
        username: String,
        avatar: String,
        address: SuiAddress,
        sui_coin: Option<Coin>,
        game_token: Option<Coin>,
        admin: Option<SuiAdminObject>,
        hero: Option<SuiHeroObject>,
        pet: Option<SuiPetObject>,
        bot: Option<SuiBotObject>,
    ) -> Self {
        UserGameState {
            id: UserId(id as u64),
            username,
            avatar,
            address: address.to_string(),
            sui_coin,
            game_token,
            admin,
            hero,
            pet,
            bot,
        }
    }

    pub async fn new_state(
        sui_client: &SuiClient,
        mm: &ModelManager,
        package: &ObjectID,
        user_info: &UserInfo,
    ) -> Result<Self> {
        let ctx = Ctx::root_ctx();
        let discord_profile = &user_info.discord;
        let wallet = &user_info.wallet;
        let address = SuiAddress::from_str(wallet.pub_key.as_str()).ok().unwrap();

        let wepet_game = WePetGame::new(sui_client.clone(), address, package.to_string().as_str());

        let sui_coins_stream = sui_client.coin_read_api().get_coins_stream(address, None);

        let sui_coin = sui_coins_stream
            // .skip_while(|c| future::ready(c.balance < 5_000_000))
            .boxed()
            .next()
            .await;

        let admin = wepet_game
            .get_sui_obj_first::<SuiAdminObject>(ADMIN_OBJECT_NAME)
            .await
            .ok();

        let pet = wepet_game
            .get_sui_obj_first::<SuiPetObject>(PET_OBJECT_NAME)
            .await
            .ok();

        // println!("pet data: \n{pet:?}\n");

        let bot = wepet_game
            .get_sui_obj_first::<SuiBotObject>(BOT_OBJECT_NAME)
            .await
            .ok();
        // println!("bot data: \n{bot:?}\n");

        let hero = wepet_game
            .get_sui_obj_first::<SuiHeroObject>(HERO_OBJECT_NAME)
            .await
            .ok();
        // println!("hero data: \n{hero:?}\n");

        Ok(UserGameState::new(
            discord_profile.discord_id,
            discord_profile.global_name.clone(),
            discord_profile.avatar.clone(),
            address,
            sui_coin,
            None,
            admin,
            hero,
            pet,
            bot,
        ))
    }

    pub fn get_game_state_board(&self) -> String {
        let mut game_state_board = String::from("----------------------------------------------\n");

        game_state_board.push_str(&format!("id: {:<70}\n", self.id));
        game_state_board.push_str(&format!("player: {:<70}\n", self.username));
        game_state_board.push_str(&format!("address: {:<70}\n", self.address,));

        if let Some(sui_coin) = &self.sui_coin {
            game_state_board.push_str(&format!(
                "balance: {:>20} {:<10}\n",
                sui_coin.balance, "SUI",
            ));
        } else {
            game_state_board.push_str(&format!("balance: {:>2}{:<10}\n", "0", "SUI"));
        }

        if let Some(game_token) = &self.game_token {
            game_state_board.push_str(&format!(
                "balance: {:>20}{:<10}\n",
                game_token.balance, game_token.coin_type,
            ));
        }

        if let Some(admin) = &self.admin {
            game_state_board.push_str(&format!(
                "Admin: {:>20}\nTotal Bot: {:>20}",
                admin.id, admin.bot_animal_created
            ));
        }

        if let Some(hero) = &self.hero {
            game_state_board.push_str(&format!(
                "hero: {:>20}\nLevel: {:<20}\n",
                hero.id, hero.level
            ));
        }

        if let Some(pet) = &self.pet {
            game_state_board.push_str(&format!(
                "pet: {:>20}\nHp: {:>3} -------- Exp: {:>3} -------- Strength: {:>3}\n",
                pet.id, pet.hp, pet.exp, pet.strength
            ));
        }

        if let Some(bot) = &self.bot {
            game_state_board.push_str(&format!(
                "bot: {:>20}\nHp: {:>5} -------- Strength: {:>5}\n",
                bot.id, bot.hp, bot.strength
            ));
        }

        game_state_board
    }
}
