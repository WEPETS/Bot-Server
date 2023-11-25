#![allow(unused)]

// region: -- Modules

// modules
mod _dev_init;
mod commands;
mod config;
mod ctx;
mod error;
mod event_handler;
mod game_state;
mod log;
mod middlewares;
mod models;
mod pwd;
mod routes;
mod store;
mod sui_call;
mod token;
mod utils;

// re-exports
pub use self::error::{Error, Result};
pub use config::get_config;
use dotenvy::dotenv;
use sui_keys::keystore::{FileBasedKeystore, Keystore};
use tokio::try_join;
use tracing::field::debug;

use crate::event_handler::Handler;
// imports
use crate::middlewares::{mw_ctx_resolve::mw_ctx_resolve, mw_reponse_map::mw_reponse_map};
use crate::routes::routes_static;
use anyhow;
use axum::{middleware, Router};
use models::ModelManager;
use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use sui_sdk::types::base_types::{ObjectID, SuiAddress};
use sui_sdk::{SuiClient, SuiClientBuilder};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

// endregion: -- Modules

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // tracing debug
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // get app config
    let config = get_config();

    // dev only
    if config.ENVIRONMENT == "development" {
        _dev_init::init_db().await;
    }

    // model - store layer (DB)
    let mm = ModelManager::new().await?;

    // route defination
    let routes = Router::new()
        .merge(routes::routes(mm.clone()))
        .layer(middleware::map_response(mw_reponse_map))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .fallback_service(routes_static::serve_dir());

    // Discord bot setup
    let discord_bot_task = tokio::spawn(async move {
        // sui client and discord client definition
        let discord_token = &config.DISCORD_TOKEN;

        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let sui_client = SuiClientBuilder::default().build_devnet().await.unwrap();
        let default_address = SuiAddress::from_str(&config.SUI_CLIENT_ADDRESS).unwrap_or_default();
        let keystore_path = Path::new("/home/ganzzi/.sui/sui_config/sui.keystore");
        let mut keystore =
            Keystore::from(FileBasedKeystore::new(&keystore_path.to_path_buf()).unwrap());

        let handler = Handler {
            sui_client,
            package_id: ObjectID::from_str(config.PACKAGE.as_str()).unwrap(),
            default_address,
            config,
            mm: mm.clone(),
            keystore,
        };

        let mut discord_client = Client::builder(&discord_token, intents)
            .event_handler(handler)
            .await
            .expect("Error creating Discord client");

        if let Err(why) = discord_client.start_shards(2).await {
            debug!("Discord client error: {:?}", why);
        }
    });

    // Axum server setup
    let axum_server_task = tokio::spawn(async {
        // run the server in local
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        let axum_server = axum::Server::bind(&addr)
            .serve(routes.into_make_service())
            .await
            .expect("Failed to start Axum server");

        info!("Axum server listening on: {}", addr);
    });

    // Try to join both tasks concurrently
    if let Err(e) = try_join!(discord_bot_task, axum_server_task) {
        debug!("Error joining tasks: {:?}", e);
    }

    Ok(())
}
