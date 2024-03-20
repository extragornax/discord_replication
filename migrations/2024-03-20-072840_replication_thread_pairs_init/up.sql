CREATE TABLE public.replication_thread_pairs
(
    id          bigserial                           NOT NULL,
    from_guild  bigint                              NOT NULL,
    from_thread bigint                              NOT NULL,
    to_guild    bigint                              NOT NULL,
    to_thread   bigint                              NOT NULL,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT replication_thread_pairs_pk PRIMARY KEY (id)
);
