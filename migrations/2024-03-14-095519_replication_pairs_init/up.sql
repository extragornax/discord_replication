CREATE TABLE replications_pairs
(
    id           BIGSERIAL PRIMARY KEY,
    from_guild   BIGINT,
    from_channel BIGINT,
    to_guild     BIGINT,
    to_channel   BIGINT,
    created_at   TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
