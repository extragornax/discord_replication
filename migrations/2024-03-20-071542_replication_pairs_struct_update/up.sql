ALTER TABLE public.replications_pairs RENAME TO replications_forum_pairs;
ALTER TABLE public.replications_forum_pairs RENAME COLUMN from_channel TO from_forum;
ALTER TABLE public.replications_forum_pairs RENAME COLUMN to_channel TO to_forum;
