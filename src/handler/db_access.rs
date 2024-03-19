use crate::{
    database::DBAccessManager,
    errors::AppError,
    schema::replications_pairs::from_guild,
};
use chrono::{NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::errors::ErrorType;
use crate::log::{write_debug_log, write_error_log};
use crate::schema::{replications_pairs, replications_reply};

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct ReplicationPair {
    pub id: i64,
    pub from_guild: i64,
    pub from_channel: i64,
    pub to_guild: i64,
    pub to_channel: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "replications_pairs"]
pub struct ReplicationPairData {
    pub from_guild: i64,
    pub from_channel: i64,
    pub to_guild: i64,
    pub to_channel: i64,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct ReplicationReply {
    pub id: i64,
    pub responded: bool,
    pub status: String,
    pub guild_id: i64,
    pub created_at: NaiveDateTime,
    pub channel_id: i64,
    pub replication_pairs: i64,
    pub message_id: Option<i64>,
    pub message_owner: i64,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[table_name = "replications_reply"]
pub struct ReplicationReplyData {
    pub responded: bool,
    pub status: String,
    pub guild_id: i64,
    pub channel_id: i64,
    pub replication_pairs: i64,
    pub message_id: Option<i64>,
    pub message_owner: i64,
}

impl DBAccessManager {
    pub fn get_active_replication_pair(&self, _guild_id: i64, _channel_id: i64) -> Result<Vec<ReplicationPair>, AppError> {
        use crate::schema::replications_pairs::dsl::*;
        use crate::schema::replications_pairs;
        use crate::schema::replications_reply::dsl::*;
        use crate::schema::replications_reply;

        replications_pairs
            .inner_join(replications_reply.on(replications_reply::replication_pairs.eq(replications_pairs::id)))
            .filter(replications_reply::status.eq("active"))
            .filter(from_guild.eq(_guild_id).and(from_channel.eq(_channel_id)))
            .select((
                replications_pairs::id,
                replications_pairs::from_guild,
                replications_pairs::from_channel,
                replications_pairs::to_guild,
                replications_pairs::to_channel,
                replications_pairs::created_at,
            ))
            .get_results(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while retrieving ReplicationPair"))
    }

    pub fn get_replication_pair(&self, _guild_id: i64, _channel_id: i64) -> Result<Vec<ReplicationPair>, AppError> {
        use crate::schema::replications_pairs::dsl::*;

        replications_pairs
            .filter(from_guild.eq(_guild_id).and(from_channel.eq(_channel_id)))
            .get_results(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while retrieving ReplicationPair"))
    }

    pub fn create_replication_pair(&self, dto: ReplicationPairData) -> Result<ReplicationPair, AppError> {
        diesel::insert_into(replications_pairs::table)
            .values(&dto)
            .get_result(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while creating ReplicationPair"))
    }

    pub fn get_replication_reply(&self, _guild_id: i64, _channel_id: i64) -> Result<ReplicationReply, AppError> {
        use crate::schema::replications_reply::dsl::*;

        replications_reply
            .filter(guild_id.eq(_guild_id).and(channel_id.eq(_channel_id)))
            .first(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while retrieving ReplicationReply"))
    }

    pub fn get_replication_reply_full(&self, _guild_id: i64, _channel_id: i64, _message_id: i64) -> Result<ReplicationReply, AppError> {
        use crate::schema::replications_reply::dsl::*;

        replications_reply
            .filter(guild_id.eq(_guild_id).and(channel_id.eq(_channel_id)).and(message_id.eq(_message_id)))
            .first(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while retrieving ReplicationReply"))
    }

    pub fn create_replication_reply(&self, dto: ReplicationReplyData) -> Result<ReplicationReply, AppError> {
        diesel::insert_into(replications_reply::table)
            .values(&dto)
            .get_result(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while creating ReplicationReply"))
    }

    pub fn update_replication_reply_status(&self, _guild_id: i64, _channel_id: i64, _responded: bool, _status: String) -> Result<ReplicationReply, AppError> {
        use crate::schema::replications_reply::dsl::*;

        let updated = diesel::update(replications_reply.filter(guild_id.eq(_guild_id).and(channel_id.eq(_channel_id))))
            .set((
                responded.eq(_responded),
                status.eq(_status),
            ))
            .execute(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while updating ReplicationReply"))?;

        if updated == 0 {
            return Err(AppError::new("ReplicationReply not found", ErrorType::NotFound));
        }

        self.get_replication_reply(_guild_id, _channel_id)
    }

    pub fn update_replication_reply_message_id(&self, _guild_id: i64, _channel_id: i64, _message_id: Option<i64>) -> Result<ReplicationReply, AppError> {
        use crate::schema::replications_reply::dsl::*;

        let updated = diesel::update(replications_reply.filter(guild_id.eq(_guild_id).and(channel_id.eq(_channel_id))))
            .set((
                message_id.eq(_message_id),
            ))
            .execute(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while updating ReplicationReply"))?;

        if updated == 0 {
            return Err(AppError::new("ReplicationReply not found", ErrorType::NotFound));
        }

        self.get_replication_reply(_guild_id, _channel_id)
    }
}
