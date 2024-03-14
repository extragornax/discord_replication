use serenity::all::{GuildId, PartialGuildChannel};
use serenity::all::{CacheHttp, ChannelId, ChannelType, Context, EventHandler, GuildChannel, Message, MessageId, Reaction, Ready};
use serenity::async_trait;
use crate::handler::Handler;

impl Handler {
    pub fn new(pool: crate::database::PgPool) -> Self {
        crate::handler::Handler { pool }
    }

    pub fn get_access(&self) -> Result<crate::database::DBAccessManager, crate::errors::AppError> {
        match self.pool.get() {
            Ok(conn) => Ok(crate::database::DBAccessManager::new(conn)),
            Err(err) => Err(crate::errors::AppError {
                err_type: crate::errors::ErrorType::Internal,
                message: format!("Error getting connection from pool: {}", err.to_string()),
            }),
        }
    }
}

#[async_trait]
impl EventHandler for crate::handler::Handler {
    async fn channel_create(&self, ctx: Context, channel: GuildChannel) {
        crate::log::write_info_log(format!("Channel created: {} of king {:?}", channel.name, channel.kind));

        if channel.kind == ChannelType::Text {
            if let Err(why) = channel.id.say(&ctx.http, "Welcome!").await {
                crate::log::write_error_log(format!("Error sending message: {why:?}"));
            }
        }
    }
    /*
    #[cfg(feature = "cache")]
        {
            if let Some(cache) = cache_http.cache() {
                crate::utils::user_has_perms_cache(cache, self.id, Permissions::MANAGE_CHANNELS)?;
            }
        }

        let channel = self.id.delete(cache_http.http()).await?;
        channel.guild().ok_or(Error::Model(ModelError::InvalidChannelType))
     */

    async fn cache_ready(&self, ctx: Context, _: Vec<GuildId>) {
        crate::log::write_info_log(format!("{} unknown members", ctx.cache.unknown_members()));
    }

    async fn channel_delete(&self, _ctx: Context, channel: GuildChannel, messages: Option<Vec<Message>>) {
        match _ctx.cache() {
            Some(cache) => {
                let guild_name = channel.guild(&cache).map_or(String::from("Unknown"), |g| g.name.clone());

                crate::log::write_info_log(format!("Channel deleted: {} in server {}", channel.name, guild_name));
            }
            None => {
                crate::log::write_info_log(format!("Channel deleted: {} in server id {}", channel.name, channel.guild_id));
            }
        }
    }

    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        crate::log::write_info_log(format!("MESSAGE: {}: {}", msg.author.name, msg.content));
        crate::log::write_info_log(format!("New message in {:?} from {}", msg.channel_id, msg.author.name));

        // match msg.content {
        //     x if x.starts_with("!ping") => {
        //         if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
        //             write_error_log(format!("Error sending message: {why:?}"));
        //         }
        //     }
        //     x if x.starts_with("!link") => {
        //         if let Err(why) = msg.channel_id.say(&ctx.http, "Hello!").await {
        //             write_error_log(format!("Error sending message: {why:?}"));
        //         }
        //     }
        //     _ => {}
        // }
        //
        // match msg.thread {
        //     Some(thread) => {
        //         write_info_log(format!("Thread: {}", thread));
        //     }
        //     None => {
        //         write_info_log("No thread".to_string());
        //     }
        // }
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        crate::log::write_info_log(format!("Reaction added: {:?}", add_reaction));
    }

    async fn reaction_remove(&self, ctx: Context, remove_reaction: Reaction) {
        crate::log::write_info_log(format!("Reaction removed: {:?}", remove_reaction));
    }

    async fn reaction_remove_all(&self, ctx: Context, channel_id: ChannelId, message_id: MessageId) {
        crate::log::write_info_log(format!("All reactions removed from message: {} in channel: {}", message_id, channel_id));
    }

    async fn thread_delete(&self, ctx: Context, thread: PartialGuildChannel, channel: Option<GuildChannel>) {
        // drop replication
        // Notify ?
        crate::log::write_info_log(format!("Thread deleted: {}", channel.unwrap_or_default().name));
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, context: Context, ready: Ready) {
        crate::log::write_info_log(format!("{} is connected!", ready.user.name));

        let guilds = context.cache.guilds().len();
        crate::log::write_info_log(format!("Guilds in the Cache: {}", guilds));
    }

    // async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
    //     write_info_log(format!("Thread created: {}", thread.name));
    //
    //     if let Err(why) = thread.id.say(&ctx.http, "Welcome!").await {
    //         write_error_log(format!("Error sending message: {why:?}"));
    //     }
    // }
}
