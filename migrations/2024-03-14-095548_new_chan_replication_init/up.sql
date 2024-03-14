CREATE TABLE replications_reply
(
    id             BIGSERIAL PRIMARY KEY,
    responded      BOOLEAN,
    status         VARCHAR(255),
    replication_id BIGINT REFERENCES replications_pairs (id),
    created_at     TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
