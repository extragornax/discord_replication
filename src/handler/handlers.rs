use std::num::NonZeroU64;
use std::sync::Arc;
use serenity::all::{GuildId, PartialGuildChannel};
use serenity::all::{CacheHttp, ChannelId, ChannelType, Context, EventHandler, GuildChannel, Message, MessageId, Reaction, Ready};
use serenity::async_trait;
use crate::DbHandler;
use crate::handler::db_access::ReplicationReplyData;
use crate::handler::Handler;
use crate::log::{write_error_log, write_info_log};

const UP_EMOJI: char = '👍';
const DOWN_EMOJI: char = '👎';

impl Handler {
    pub fn new(pool: Arc<DbHandler>) -> Self {
        crate::handler::Handler { pool }
    }

    pub fn get_access(&self) -> Result<crate::database::DBAccessManager, crate::errors::AppError> {
        match self.pool.pool.get() {
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
    // async fn channel_create(&self, ctx: Context, channel: GuildChannel) {
    //     write_info_log(format!("Channel created: {} of king {:?}", channel.name, channel.kind));
    //
    //     if channel.kind == ChannelType::Text {
    //         if let Err(why) = channel.id.say(&ctx.http, "Welcome!").await {
    //             crate::log::write_error_log(format!("Error sending message: {why:?}"));
    //         }
    //
    //         if channel.parent_id.unwrap_or_default() == 1216723543929258044 {
    //             if let Err(why) = channel.id.say(&ctx.http, "Welcome to the special channel! 1216723543929258044").await {
    //                 crate::log::write_error_log(format!("Error sending message: {why:?}"));
    //             }
    //         }
    //     }
    // }
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
        write_info_log(format!("{} unknown members", ctx.cache.unknown_members()));
    }

    async fn channel_delete(&self, _ctx: Context, channel: GuildChannel, messages: Option<Vec<Message>>) {
        match _ctx.cache() {
            Some(cache) => {
                let guild_name = channel.guild(&cache).map_or(String::from("Unknown"), |g| g.name.clone());

                write_info_log(format!("Channel deleted: {} in server {}", channel.name, guild_name));
            }
            None => {
                write_info_log(format!("Channel deleted: {} in server id {}", channel.name, channel.guild_id));
            }
        }
    }

    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        write_info_log(format!("MESSAGE: {}: {}", msg.author.name, msg.content));
        write_info_log(format!("New message in {:?} from {}", msg.channel_id, msg.author.name));

        if msg.author.bot {
            return;
        }

        let _db_access = match self.pool.pool.get() {
            Ok(conn) => crate::database::DBAccessManager::new(conn),
            Err(err) => {
                write_error_log(format!("Error getting connection from pool: {}", err.to_string()));
                return;
            }
        };

        match _db_access.get_active_replication_pair(msg.guild_id.unwrap_or_default().get() as i64, msg.channel_id.get() as i64) {
            Ok(found) => {
                msg.channel_id.say(&ctx.http, format!("Replicated message: {:?}", msg.content)).await;

                for f in found {
                    // ctx.http.create_message(f.to_channel).content(format!("Replicated message: {:?}", msg.content)).await;
                    let as_nonzerou64_guild = f.to_guild as u64;

                    let guild = match ctx.cache.guild(as_nonzerou64_guild) {
                        Some(guild) => guild.clone(),
                        None => {
                            write_error_log(format!("Guild not found: {}", f.to_guild));
                            return;
                        }
                    };
                    let u64_channel = f.to_channel as u64;
                    let as_nonzerou64_channel = NonZeroU64::new(u64_channel).unwrap();
                    let chan: ChannelId = as_nonzerou64_channel.into();

                    let channel = match guild.channels.get(&chan) {
                        Some(channel) => {
                            channel
                        }
                        None => {
                            write_error_log(format!("Channel not found: {}", f.to_channel));
                            return;
                        }
                    };

                    match channel.id.say(&ctx.http, format!("Replicated message: {:?}", msg.content)).await {
                        Ok(_) => {
                            write_info_log(format!("Replicated message: {:?}", msg.content));
                        }
                        Err(why) => {
                            write_error_log(format!("Error sending message: {why:?}"));
                        }
                    }
                }
            }
            _ => {}
        }

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
        write_info_log(format!("Reaction added: {:?}", add_reaction));

        //ignore if bot
        match add_reaction.member {
            Some(user) => {
                if user.user.bot {
                    return;
                }
            }
            _ => {}
        }
        // add_reaction.channel_id.say(&ctx.http, format!("Reaction added: {:?}", add_reaction)).await;

        let _db_access = match self.pool.pool.get() {
            Ok(conn) => crate::database::DBAccessManager::new(conn),
            Err(err) => {
                write_error_log(format!("Error getting connection from pool: {}", err.to_string()));
                return;
            }
        };

        let guild_id = add_reaction.guild_id.unwrap_or_default().get() as i64;
        let channel_id = add_reaction.channel_id.get() as i64;
        let message_id = add_reaction.message_id.get() as i64;
        let user_id = add_reaction.user_id.unwrap_or_default().get() as i64;

        match _db_access.get_replication_reply_full(guild_id, channel_id, message_id) {
            Ok(data) => {
                write_info_log(format!("Replication reply found: {:?}", data));
                write_info_log(format!("Replication reply found: {:?}", data));

                if data.message_owner == user_id {
                    write_info_log("Message owner".to_string());
                    let _ = add_reaction.channel_id.say(&ctx.http, "Message owner").await;

                    if add_reaction.emoji.unicode_eq(format!("{UP_EMOJI}").as_str()) {
                        write_info_log("UP_EMOJI".to_string());
                        // let _ = add_reaction.channel_id.say(&ctx.http, format!("UP_EMOJI -> {}", add_reaction.emoji)).await;

                        let _db_access = match self.pool.pool.get() {
                            Ok(conn) => crate::database::DBAccessManager::new(conn),
                            Err(err) => {
                                write_error_log(format!("Error getting connection from pool: {}", err.to_string()));
                                return;
                            }
                        };

                        _db_access.update_replication_reply_status(guild_id, channel_id, true, "active".to_string());
                        todo!("Add pair channel in other server");
                        todo!("Save pair to handle it on message");
                    } else if add_reaction.emoji.unicode_eq(format!("{DOWN_EMOJI}").as_str()) {
                        write_info_log("DOWN_EMOJI".to_string());
                        // let _ = add_reaction.channel_id.say(&ctx.http, format!("DOWN_EMOJI -> {}", add_reaction.emoji)).await;
                        let _ = _db_access.update_replication_reply_status(guild_id, channel_id, true, "inactive".to_string());
                    } else {
                        write_info_log("Not a valid reaction".to_string());
                        let _ = add_reaction.channel_id.say(&ctx.http, format!("Not a valid reaction -> {}", add_reaction.emoji)).await;
                    }

                    return;
                } else {
                    write_info_log("Not message owner".to_string());
                    let _ = add_reaction.channel_id.say(&ctx.http, "Not message owner").await;
                }
            }
            Err(err) => {
                write_error_log(format!("Error getting replication reply: {}", err.message));
                let _ = add_reaction.channel_id.say(&ctx.http, format!("Error getting replication reply: {}", err.message)).await;
            }
        }
    }

    async fn reaction_remove(&self, ctx: Context, remove_reaction: Reaction) {
        write_info_log(format!("Reaction removed: {:?}", remove_reaction));
    }

    async fn reaction_remove_all(&self, ctx: Context, channel_id: ChannelId, message_id: MessageId) {
        write_info_log(format!("All reactions removed from message: {} in channel: {}", message_id, channel_id));
    }

    async fn thread_delete(&self, ctx: Context, thread: PartialGuildChannel, channel: Option<GuildChannel>) {
        // drop replication
        // Notify ?
        write_info_log(format!("Thread deleted: {}", channel.unwrap_or_default().name));
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, context: Context, ready: Ready) {
        write_info_log(format!("{} is connected!", ready.user.name));

        let guilds = context.cache.guilds().len();
        write_info_log(format!("Guilds in the Cache: {}", guilds));
    }

    async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
        write_info_log(format!("Thread created: {} ({})", thread.name, ctx.shard_id));

        // if let Err(why) = thread.id.say(&ctx.http, "Welcome!").await {
        //     write_error_log(format!("Error sending message: {why:?}"));
        // }

        let _db_access = match self.pool.pool.get() {
            Ok(conn) => crate::database::DBAccessManager::new(conn),
            Err(err) => {
                write_error_log(format!("Error getting connection from pool: {}", err.to_string()));
                return;
            }
        };

        let guild_id = thread.guild_id.get() as i64;
        let parent_id = thread.parent_id.unwrap_or_default().get() as i64;

        let _check_pair = _db_access.get_replication_pair(guild_id, parent_id);
        match _check_pair {
            Ok(data) => {
                let parsed: Vec<(i64, i64, i64)> = data.iter().map(|i| {
                    (i.id, i.to_guild, i.to_channel)
                }).collect();

                for (replication_id, to_guild, to_channel) in parsed {
                    let _check = _db_access.get_replication_reply(guild_id, thread.id.get() as i64);
                    if _check.is_ok() {
                        break;
                    }

                    match thread.owner_id {
                        Some(owner) => {
                            write_info_log(format!("Thread owner: {}", owner));
                            let _dto = ReplicationReplyData {
                                responded: false,
                                status: "inactive".to_string(),
                                guild_id: thread.guild_id.get() as i64,
                                channel_id: thread.id.get() as i64,
                                replication_pairs: replication_id,
                                message_id: None,
                                message_owner: owner.get() as i64,
                            };

                            let _check = _db_access.get_replication_reply(_dto.guild_id, _dto.channel_id);
                            if _check.is_ok() {
                                break;
                            }

                            match _db_access.create_replication_reply(_dto) {
                                Ok(d) => {
                                    write_info_log("Replication reply created".to_string());
                                }
                                Err(err) => {
                                    write_error_log(format!("Error creating replication reply: {}", err.message));
                                    let _ = thread.id.say(&ctx.http, format!("Error creating replication reply: {}", err.message)).await;
                                }
                            };

                            match thread.id.say(&ctx.http, format!("Do you want to pair with https://discord.com/channels/{}/{} ?", to_guild, to_channel)).await {
                                Ok(_msg_tmp) => {
                                    let _ = _db_access.update_replication_reply_message_id(thread.guild_id.get() as i64, _msg_tmp.channel_id.get() as i64, Some(_msg_tmp.id.get() as i64));

                                    match _msg_tmp.react(&ctx.http, UP_EMOJI).await {
                                        Ok(_) => {
                                            write_info_log(format!("Reacted with {UP_EMOJI}"));
                                        }
                                        Err(why) => {
                                            write_error_log(format!("Error sending message: {why:?}"));
                                            let _ = thread.id.say(&ctx.http, format!("Error sending message: {why:?}")).await;
                                        }
                                    }
                                    match _msg_tmp.react(&ctx.http, DOWN_EMOJI).await {
                                        Ok(_) => {
                                            write_info_log(format!("Reacted with {DOWN_EMOJI}"));
                                        }
                                        Err(why) => {
                                            write_error_log(format!("Error sending message: {why:?}"));
                                            let _ = thread.id.say(&ctx.http, format!("Error sending message: {why:?}")).await;
                                        }
                                    };

                                    write_info_log(format!("Do you want to pair with https://discord.com/channels/{}/{} ?", to_guild, to_channel));
                                }
                                Err(why) => {
                                    write_error_log(format!("Error sending message: {why:?}"));
                                    let _ = thread.id.say(&ctx.http, format!("Error sending message: {why:?}")).await;
                                }
                            };
                        }
                        None => {
                            write_info_log("No thread owner".to_string());
                            let _ = thread.id.say(&ctx.http, "No thread owner").await;
                        }
                    }
                }
            }
            Err(err) => {
                write_error_log(format!("Error getting replication pair: {}", err.message));
                let _ = thread.id.say(&ctx.http, format!("Error getting replication pair: {}", err.message)).await;
            }
        }
    }
}
