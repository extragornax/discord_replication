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

ALTER TABLE public.replication_thread_pairs
    ADD replication_reply_id int8 NOT NULL REFERENCES public.replications_reply (id) ON DELETE CASCADE ON UPDATE CASCADE;
