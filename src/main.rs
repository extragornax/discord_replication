#![recursion_limit = "256"]
extern crate openssl;
// DO NOT MOVE THIS LINE
#[macro_use]
extern crate diesel;

mod handler;
mod log;
mod database;
mod errors;
mod schema;

use serenity::{
    http::Http,
    all::{StandardFramework, UserId},
    prelude::*,
    gateway::ShardManager,
};
use std::{
    collections::{HashMap, HashSet},
    env,
    sync::Arc,
};
use crate::{
    database::{get_pg_pool, PgPool},
    handler::{
        commands::create_framework,
        Handler,
        hooks::CommandCounter,
    },
    log::{write_error_log, write_info_log},
};
use crate::database::DBAccessManager;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

pub struct DbHandler {
    pub pool: PgPool,
}

impl serenity::prelude::TypeMapKey for DbHandler {
    type Value = Arc<DbHandler>;
}

impl DbHandler {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn mut_as_db_access(&self) -> DBAccessManager {
        match self.pool.get() {
            Ok(conn) => DBAccessManager::new(conn),
            Err(err) => panic!("Error getting connection from pool: {}", err.to_string()),
        }
    }
}

fn handle_database_init() -> DbHandler {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env not set");

    let db_pool: PgPool = get_pg_pool(database_url.as_str());

    DbHandler::new(db_pool)
}


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

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

    let db_handler = Arc::new(handle_database_init());

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler::new(db_handler.clone()))
        .framework(framework)
        .type_map_insert::<CommandCounter>(HashMap::default())
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        let shard_manager = client.shard_manager.clone();

        write_info_log(format!("Inserting ShardManager into data map -> {:?}", shard_manager));
        data.insert::<ShardManagerContainer>(shard_manager);
        data.insert::<DbHandler>(db_handler.clone());
    }

    // Here we clone a lock to the Shard Manager, and then move it into a new thread. The thread
    // will unlock the manager and print shards' status on a loop.
    // let manager = client.shard_manager.clone();

    // tokio::spawn(async move {
    //     loop {
    //         sleep(Duration::from_secs(30)).await;
    //
    //         let shard_runners = manager.runners.lock().await;
    //
    //         for (id, runner) in shard_runners.iter() {
    //             println!(
    //                 "Shard ID {} is {} with a latency of {:?}",
    //                 id, runner.stage, runner.latency,
    //             );
    //         }
    //     }
    // });

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        write_error_log(format!("Client error: {why:?}"));
    }
}
