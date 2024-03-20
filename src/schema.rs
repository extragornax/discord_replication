table! {
    replication_thread_pairs (id) {
        id -> Int8,
        from_guild -> Int8,
        from_thread -> Int8,
        to_guild -> Int8,
        to_thread -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    replications_forum_pairs (id) {
        id -> Int8,
        from_guild -> Int8,
        from_forum -> Int8,
        to_guild -> Int8,
        to_forum -> Int8,
        created_at -> Timestamp,
    }
}

table! {
    replications_reply (id) {
        id -> Int8,
        responded -> Bool,
        status -> Varchar,
        guild_id -> Int8,
        created_at -> Timestamp,
        channel_id -> Int8,
        replication_pairs -> Int8,
        message_id -> Nullable<Int8>,
        message_owner -> Int8,
    }
}

joinable!(replications_reply -> replications_forum_pairs (replication_pairs));

allow_tables_to_appear_in_same_query!(
    replication_thread_pairs,
    replications_forum_pairs,
    replications_reply,
);
