CREATE TABLE IF NOT EXISTS prefixes(
    guild_id BIGINT PRIMARY KEY,
    prefix VARCHAR(5) NOT NULL,
    CONSTRAINT fk_guilds FOREIGN KEY(guild_id) REFERENCES guilds(guild_id) ON
    DELETE
        CASCADE
);