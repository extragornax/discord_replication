use std::any::Any;
use serenity::all::{ChannelType, Context, EventHandler, GuildChannel, Message, Reaction, Ready};
use serenity::async_trait;
use crate::log::{write_error_log, write_info_log};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn channel_create(&self, ctx: Context, channel: GuildChannel) {
        write_info_log(format!("Channel created: {} of king {:?}", channel.name, channel.kind));

        if channel.kind == ChannelType::Text {
            if let Err(why) = channel.id.say(&ctx.http, "Welcome!").await {
                write_error_log(format!("Error sending message: {why:?}"));
            }
        }
    }

    async fn channel_delete(&self, _ctx: Context, channel: GuildChannel, messages: Option<Vec<Message>>) {
        let guild_name = channel.guild().as_ref().map_or("Unknown", |g| g.name.as_str());

        write_info_log(format!("Channel deleted: {} in server {}", channel.name, guild_name));
    }

    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an authentication error, or lack
            // of permissions to post in the channel, so log to stdout when some error happens,
            // with a description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                write_error_log(format!("Error sending message: {why:?}"));
            }
        }
        write_info_log(format!("MESSAGE: {}: {}", msg.author.name, msg.content));
        write_info_log(format!("New message in {:?} from {}", msg.channel_id, msg.author.name));
        match msg.thread {
            Some(thread) => {
                write_info_log(format!("Thread: {}", thread));
            }
            None => {
                write_info_log("No thread".to_string());
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        write_info_log(format!("Reaction added: {:?}", add_reaction));
    }

    async fn reaction_remove(&self, ctx: Context, remove_reaction: Reaction) {
        write_info_log(format!("Reaction removed: {:?}", remove_reaction));
    }

    async fn reaction_remove_all(&self, ctx: Context, channel_id: u64, message_id: u64) {
        write_info_log(format!("All reactions removed from message: {} in channel: {}", message_id, channel_id));
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        write_info_log(format!("{} is connected!", ready.user.name));
    }

    // async fn thread_create(&self, ctx: Context, thread: GuildChannel) {
    //     write_info_log(format!("Thread created: {}", thread.name));
    //
    //     if let Err(why) = thread.id.say(&ctx.http, "Welcome!").await {
    //         write_error_log(format!("Error sending message: {why:?}"));
    //     }
    // }
}
