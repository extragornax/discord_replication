use std::num::NonZeroU64;
use std::sync::Arc;
use serde::Serialize;
use serenity::all::{CreateMessage, Guild, GuildId, PartialGuildChannel};
use serenity::all::{CacheHttp, ChannelId, ChannelType, Context, EventHandler, GuildChannel, Message, MessageId, Reaction, Ready};
use serenity::async_trait;
use serenity::builder::{CreateChannel, CreateForumPost, CreateThread};
use crate::DbHandler;
use crate::handler::db_access::{ReplicationReplyData, ReplicationThreadPairData};
use crate::handler::Handler;
use crate::log::{write_error_log, write_info_log};

const UP_EMOJI: char = '👍';
const DOWN_EMOJI: char = '👎';
const ROCKET_EMOJI: char = '🚀';
const BOMB_EXPLODED_EMOJI: char = '💥';

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
        if msg.author.bot {
            return;
        }

        write_info_log(format!("MESSAGE: {}: {}", msg.author.name, msg.content));
        write_info_log(format!("New message in {:?} from {}", msg.channel_id, msg.author.name));

        let _db_access = match self.pool.pool.get() {
            Ok(conn) => crate::database::DBAccessManager::new(conn),
            Err(err) => {
                write_error_log(format!("Error getting connection from pool: {}", err.to_string()));
                return;
            }
        };

        // let message_channgel_url = format!("https://discord.com/channels/{}/{}", msg.guild_id.unwrap_or_default(), msg.channel_id);
        // msg.channel_id.say(&ctx.http, format!("New message in {:?} from {} -> {}", msg.channel_id, msg.author.name, message_channgel_url)).await;

        match _db_access.get_replication_thread_pairs(msg.guild_id.unwrap_or_default().get() as i64, msg.channel_id.get() as i64) {
            Ok(found) => {
                // let _ = msg.channel_id.say(&ctx.http, format!("Replicated message: {:?} -> {:?}", msg.content, found)).await;
                // let _ = msg.channel_id.say(&ctx.http, format!("Guid {} channel id {}", msg.guild_id.clone().unwrap_or_default(), msg.channel_id)).await;

                let message_owner_name = msg.author.name.clone();
                let mut message_without_quotes = msg.content.clone();
                // message_without_quotes.remove(0);
                // message_without_quotes.pop();

                for f in found {
                    // ctx.http.create_message(f.to_channel).content(format!("Replicated message: {:?}", msg.content)).await;
                    // msg.channel_id.say(&ctx.http, format!("Replicating to guild {} channel {}", f.to_guild, f.to_thread)).await;
                    // let distant_url = format!("https://discord.com/channels/{}/{}", f.to_guild, f.to_thread);
                    // msg.channel_id.say(&ctx.http, format!("Distant URL: {}", distant_url)).await;

                    let as_nonzerou64_guild = f.to_guild as u64;

                    let guild = match ctx.cache.guild(as_nonzerou64_guild) {
                        Some(guild) => guild.clone(),
                        None => {
                            write_error_log(format!("Guild not found: {}", f.to_guild));
                            return;
                        }
                    };
                    let u64_thread = f.to_thread as u64;
                    // let as_nonzerou64_channel = NonZeroU64::new(u64_thread).unwrap();
                    // let threa: ChannelId = as_nonzerou64_channel.into();

                    let threads_list = guild.threads;
                    let distant_thread = match threads_list.into_iter().filter(|t| t.id == u64_thread).collect::<Vec<GuildChannel>>().first() {
                        Some(thread) => thread.clone(),
                        None => {
                            write_error_log(format!("Thread not found: {}", f.to_thread));
                            return;
                        }
                    };

                    match distant_thread.id.say(&ctx.http, format!("`{}`: {}", message_owner_name, message_without_quotes)).await {
                        Ok(_) => {
                            write_info_log(format!("Replicated message: {:?}", msg.content));
                            // let _ = msg.channel_id.say(&ctx.http, format!("Found thread named: {:?} in guild {}", distant_thread.name, distant_thread.guild_id)).await;
                            let _ = msg.react(&ctx.http, ROCKET_EMOJI).await;
                        }
                        Err(why) => {
                            write_error_log(format!("Error sending message: {why:?}"));
                            let _ = msg.react(&ctx.http, BOMB_EXPLODED_EMOJI).await;
                        }
                    }
                }
            }
            _ => {
                write_info_log("No replication pair found".to_string());
            }
        }
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
            Ok(replication_reply_data) => {
                write_info_log(format!("Replication reply found: {:?}", replication_reply_data));

                if replication_reply_data.message_owner == user_id {
                    write_info_log("Message owner".to_string());
                    let _ = add_reaction.channel_id.say(&ctx.http, "Message owner").await;
                    let _ = add_reaction.channel_id.delete_message(&ctx.http, add_reaction.message_id).await;

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

                        let _ = _db_access.update_replication_reply_status(guild_id, channel_id, true, "active".to_string());

                        let current_thread_name = add_reaction.channel_id.name(&ctx.http).await.unwrap_or("Replicated thread".to_string());

                        let parent_forum = _db_access.get_parent_forum_from_message_id(guild_id, add_reaction.message_id.into()).unwrap_or_default();

                        let _remote_guilds = match _db_access.get_replication_forum_pair(guild_id, parent_forum) {
                            Ok(_remote_pairs) => {
                                add_reaction.channel_id.say(&ctx.http, format!("Remote pairs: {:?}", _remote_pairs)).await;

                                for r in _remote_pairs {
                                    let guild: Result<Guild, String> = match ctx.cache.guild(r.to_guild as u64) {
                                        Some(guild) => Ok(guild.clone()),
                                        _ => {
                                            write_error_log(format!("Guild not found: {}", r.to_guild));

                                            Err("Guild not found".to_string())
                                        }
                                    };

                                    let guild = match guild {
                                        Ok(guild) => guild,
                                        Err(err) => {
                                            write_error_log(format!("Error getting guild: {}", err));
                                            add_reaction.channel_id.say(&ctx.http, format!("Error getting guild: {}", err)).await;
                                            return;
                                        }
                                    };

                                    let u64_forum = r.to_forum as u64;
                                    let as_nonzerou64_channel = NonZeroU64::new(u64_forum).unwrap();
                                    let forum: ChannelId = as_nonzerou64_channel.into();

                                    let remote_channel = guild.channels.get(&forum).unwrap().clone();

                                    let init_message = CreateMessage::new().content(format!("FIRST MSG - {} - REPLICATED", current_thread_name));

                                    let forum_post = CreateForumPost::new(format!("{} - REPLICATED", current_thread_name), init_message);

                                    // let new_thread = remote_channel.create_thread(&ctx.http, thread_dup).await;
                                    let new_thread = remote_channel.create_forum_post(&ctx.http, forum_post).await;
                                    match new_thread {
                                        Ok(new_thread_data) => {
                                            let first = ReplicationThreadPairData {
                                                from_guild: guild_id,
                                                from_thread: channel_id,
                                                to_guild: r.to_guild,
                                                to_thread: new_thread_data.id.get() as i64,
                                                replication_reply_id: replication_reply_data.id,
                                            };
                                            let second = ReplicationThreadPairData {
                                                from_guild: r.to_guild,
                                                from_thread: new_thread_data.id.get() as i64,
                                                to_guild: guild_id,
                                                to_thread: channel_id,
                                                replication_reply_id: replication_reply_data.id,
                                            };
                                            let url_one = format!("g: {} id: {} -> https://discord.com/channels/{}/{} ?", first.from_guild, first.from_thread, first.from_guild, first.from_thread);
                                            let url_two = format!("g: {} id: {} -> https://discord.com/channels/{}/{} ?", second.from_guild, second.from_thread, second.from_guild, second.from_thread);

                                            let _ = _db_access.create_replication_thread_pair(first);
                                            let _ = _db_access.create_replication_thread_pair(second);


                                            let _ = add_reaction.channel_id.say(&ctx.http, format!("Thread created: {} in guild {} FROM {} TO {}", new_thread_data.name, guild.name, url_one, url_two)).await;
                                        }
                                        Err(e) => {
                                            write_error_log(format!("Error creating thread: {}", e));
                                            let _ = add_reaction.channel_id.say(&ctx.http, format!("Error creating thread: {}", e)).await;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        };

                        // todo!("Add pair channel in other server");
                        // todo!("Save pair to handle it on message");
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
        //ignore if bot
        match remove_reaction.member {
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

        let guild_id = remove_reaction.guild_id.unwrap_or_default().get() as i64;
        let channel_id = remove_reaction.channel_id.get() as i64;
        let message_id = remove_reaction.message_id.get() as i64;
        let user_id = remove_reaction.user_id.unwrap_or_default().get() as i64;

        match _db_access.get_replication_reply_full(guild_id, channel_id, message_id) {
            Ok(replication_reply_data) => {
                write_info_log(format!("Replication reply found: {:?}", replication_reply_data));

                if replication_reply_data.message_owner == user_id {
                    write_info_log("Message owner".to_string());
                    let _ = _db_access.delete_replication_reply(replication_reply_data.id);
                    // delete message
                    let _ = remove_reaction.channel_id.delete_message(&ctx.http, remove_reaction.message_id).await;
                }
            }
            _ => {}
        }
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

        let _db_access = match self.pool.pool.get() {
            Ok(conn) => crate::database::DBAccessManager::new(conn),
            Err(err) => {
                write_error_log(format!("Error getting connection from pool: {}", err.to_string()));
                return;
            }
        };

        let guild_id = thread.guild_id.get() as i64;
        let parent_id = thread.parent_id.unwrap_or_default().get() as i64;

        let _check_pair = _db_access.get_replication_forum_pair(guild_id, parent_id);
        match _check_pair {
            Ok(data) => {
                let parsed: Vec<(i64, i64, i64)> = data.iter().map(|i| {
                    (i.id, i.to_guild, i.to_forum)
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
