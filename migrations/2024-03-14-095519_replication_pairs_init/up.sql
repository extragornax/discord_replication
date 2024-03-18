CREATE TABLE replications_pairs
(
    id           BIGSERIAL PRIMARY KEY,
    from_guild   BIGINT,
    from_channel BIGINT,
    to_guild     BIGINT,
    to_channel   BIGINT,
    created_at   TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE public.replications_pairs
    ALTER COLUMN from_guild SET NOT NULL;
ALTER TABLE public.replications_pairs
    ALTER COLUMN from_channel SET NOT NULL;
ALTER TABLE public.replications_pairs
    ALTER COLUMN to_guild SET NOT NULL;
ALTER TABLE public.replications_pairs
    ALTER COLUMN to_channel SET NOT NULL;
ALTER TABLE public.replications_pairs
    ALTER COLUMN created_at SET NOT NULL;
