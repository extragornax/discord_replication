use serenity::all::{GuildId, PartialGuildChannel};
use std::any::Any;
use serenity::all::{CacheHttp, ChannelId, ChannelType, Context, EventHandler, GuildChannel, Message, MessageId, Reaction, Ready};
use serenity::async_trait;
use tokio::runtime::Handle;
use crate::database::{DBAccessManager, PgPool};
use crate::errors::{AppError, ErrorType};
use crate::log::{write_error_log, write_info_log};

pub mod commands;
pub mod handlers;
pub mod hooks;
mod owner;

pub struct Handler {
    pub pool: PgPool,
}
