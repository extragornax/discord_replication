#![recursion_limit = "256"]
extern crate openssl;
// DO NOT MOVE THIS LINE
#[macro_use]
extern crate diesel;

mod handler;
mod log;
mod database;
mod errors;

use serenity::http::Http;
use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;
use serenity::all::{StandardFramework, User, UserId};
use serenity::prelude::*;
use crate::database::{get_pg_pool, PgPool};
use crate::handler::commands::create_framework;
use crate::handler::Handler;
use crate::handler::hooks::CommandCounter;
use crate::log::write_error_log;
use serenity::gateway::ShardManager;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

fn handle_database_init() -> PgPool {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env not set");

    let db_pool: PgPool = get_pg_pool(database_url.as_str());

    db_pool
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    // let intents = GatewayIntents::GUILD_MESSAGES
    //     // | GatewayIntents::DIRECT_MESSAGES
    //     | GatewayIntents::MESSAGE_CONTENT
    //     | GatewayIntents::GUILD_MEMBERS
    //     | GatewayIntents::GUILD_PRESENCES
    //     | GatewayIntents::GUILD_MESSAGE_REACTIONS
    //     | GatewayIntents::GUILDS;

    let intents = GatewayIntents::all();

    let http = Http::new(&token);
    let (owners, bot_id): (HashSet<UserId>, UserId) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };


    let framework: StandardFramework = create_framework(owners, bot_id).await;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    // let mut client =
    //     Client::builder(&token, intents)
    //         .event_handler(Handler::new(handle_database_init()))
    //         .framework(framework)
    //         .type_map_insert::<CommandCounter>(HashMap::default())
    //         .await.expect("Err creating client");

    let intents = GatewayIntents::all();
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler::new(handle_database_init()))
        .framework(framework)
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        write_error_log(format!("Client error: {why:?}"));
    }
}
