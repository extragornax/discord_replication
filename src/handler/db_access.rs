use crate::{
    database::DBAccessManager,
    errors::AppError,
    schema::{
        replications_forum_pairs::from_guild,
        replications_forum_pairs,
        replications_reply,
        replication_thread_pairs,
    },
    errors::ErrorType,
    log::{write_debug_log, write_error_log},
};
use chrono::{NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::replication_thread_pairs::from_thread;

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct ReplicationForumPair {
    pub id: i64,
    pub from_guild: i64,
    pub from_forum: i64,
    pub to_guild: i64,
    pub to_forum: i64,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "replications_forum_pairs"]
pub struct ReplicationForumPairData {
    pub from_guild: i64,
    pub from_forum: i64,
    pub to_guild: i64,
    pub to_forum: i64,
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

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct ReplicationThreadPair {
    pub id: i64,
    pub from_guild: i64,
    pub from_thread: i64,
    pub to_guild: i64,
    pub to_thread: i64,
    pub created_at: NaiveDateTime,
    pub replication_reply_id: i64,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[table_name = "replication_thread_pairs"]
pub struct ReplicationThreadPairData {
    pub from_guild: i64,
    pub from_thread: i64,
    pub to_guild: i64,
    pub to_thread: i64,
    pub replication_reply_id: i64,
}

impl DBAccessManager {
    pub fn get_replication_forum_pair(&self, _guild_id: i64, _channel_id: i64) -> Result<Vec<ReplicationForumPair>, AppError> {
        use crate::schema::replications_forum_pairs::dsl::*;

        replications_forum_pairs
            .filter(from_guild.eq(_guild_id).and(from_forum.eq(_channel_id)))
            .get_results(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while retrieving ReplicationPair"))
    }

    pub fn get_replication_forum_pair_by_id(&self, _id: i64) -> Result<ReplicationForumPair, AppError> {
        use crate::schema::replications_forum_pairs::dsl::*;

        replications_forum_pairs
            .find(_id)
            .get_result(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while retrieving ReplicationPair"))
    }

    pub fn create_replication_forum_pair(&self, dto: ReplicationForumPairData) -> Result<ReplicationForumPair, AppError> {
        diesel::insert_into(replications_forum_pairs::table)
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

    pub fn delete_replication_reply(&self, _id: i64) -> Result<usize, AppError> {
        use crate::schema::replications_reply::dsl::*;

        diesel::delete(replications_reply.filter(id.eq(_id)))
            .execute(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while deleting convertionrate"))
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

    pub fn get_replication_thread_pairs(&self, _guild_id: i64, _thread_id: i64) -> Result<Vec<ReplicationThreadPair>, AppError> {
        use crate::schema::replication_thread_pairs::dsl::*;

        replication_thread_pairs
            .filter(from_guild.eq(_guild_id).and(from_thread.eq(_thread_id)))
            .get_results(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while retrieving ReplicationPair"))
    }

    pub fn create_replication_thread_pair(&self, dto: ReplicationThreadPairData) -> Result<ReplicationThreadPair, AppError> {
        diesel::insert_into(replication_thread_pairs::table)
            .values(&dto)
            .get_result(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while creating ReplicationPair"))
    }

    pub fn get_parent_forum_from_message_id(&self, _guild_id: i64, _message_id: i64) -> Result<i64, AppError> {
        use crate::schema::replications_reply::dsl::*;

        replications_reply
            .filter(guild_id.eq(_guild_id).and(message_id.eq(_message_id)))
            .first(&self.connection)
            .map_err(|err| AppError::from_diesel_err(err, "while retrieving ReplicationReply"))
            .and_then(|reply: ReplicationReply| {
                self.get_replication_forum_pair_by_id(reply.replication_pairs)
                    .map(|pair| {
                        pair.from_forum
                    })
            })
    }
}
