CREATE TABLE IF NOT EXISTS perms (
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    perm_level SMALLINT NOT NULL
);
CREATE UNIQUE INDEX idx__perms__guild_id__user_id ON perms (guild_id, user_id);