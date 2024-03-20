CREATE TABLE replications_reply
(
    id             BIGSERIAL PRIMARY KEY,
    responded      BOOLEAN,
    status         VARCHAR(255),
    replication_id BIGINT,
    created_at     TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

ALTER TABLE public.replications_reply
    ALTER COLUMN responded SET NOT NULL;
ALTER TABLE public.replications_reply
    ALTER COLUMN status SET NOT NULL;
ALTER TABLE public.replications_reply
    ALTER COLUMN replication_id SET NOT NULL;
ALTER TABLE public.replications_reply
    ALTER COLUMN created_at SET NOT NULL;

ALTER TABLE public.replications_reply ALTER COLUMN responded SET NOT NULL;
ALTER TABLE public.replications_reply ALTER COLUMN status SET NOT NULL;
ALTER TABLE public.replications_reply RENAME COLUMN replication_id TO guild_id;
ALTER TABLE public.replications_reply ALTER COLUMN guild_id SET NOT NULL;
ALTER TABLE public.replications_reply ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE public.replications_reply ADD channel_id int8 NOT NULL;
ALTER TABLE public.replications_reply ADD replication_pairs int8 NOT NULL REFERENCES replications_pairs (id);
ALTER TABLE public.replications_reply ADD message_id int8;
ALTER TABLE public.replications_reply ADD message_owner int8 NOT NULL;
