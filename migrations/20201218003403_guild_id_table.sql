CREATE TABLE IF NOT EXISTS guilds(guild_id BIGINT PRIMARY KEY);

INSERT INTO
    guilds
SELECT
    guild_id
FROM
    perms ON CONFLICT DO NOTHING;

ALTER TABLE
    perms
ADD
    CONSTRAINT fk_guilds FOREIGN KEY (guild_id) REFERENCES guilds (guild_id) ON
DELETE
    CASCADE;