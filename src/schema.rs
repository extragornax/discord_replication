table! {
    replications_pairs (id) {
        id -> Int8,
        from_guild -> Int8,
        from_channel -> Int8,
        to_guild -> Int8,
        to_channel -> Int8,
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
    }
}

allow_tables_to_appear_in_same_query!(
    replications_pairs,
    replications_reply,
);
