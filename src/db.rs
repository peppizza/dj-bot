use std::convert::TryInto;

use serenity::model::id::GuildId;
use sqlx::postgres::{PgPool, PgQueryResult};
use tracing::warn;

use crate::data::PrefixCacheInternal;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub enum UserPerm {
    Blacklisted,
    User,
    DJ,
    Admin,
}

impl From<i16> for UserPerm {
    fn from(i: i16) -> Self {
        match i {
            0 => Self::User,
            1 => Self::Blacklisted,
            2 => Self::DJ,
            3 => Self::Admin,
            _ => panic!("Can only be 0-3"),
        }
    }
}

impl Into<i16> for UserPerm {
    fn into(self) -> i16 {
        match self {
            Self::User => 0,
            Self::Blacklisted => 1,
            Self::DJ => 2,
            Self::Admin => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UserPerm;

    #[test]
    fn test_ord() {
        assert!(UserPerm::Admin > UserPerm::DJ);
        assert!(UserPerm::DJ > UserPerm::User);
        assert!(UserPerm::User > UserPerm::Blacklisted);
    }
}

pub async fn get_user_perms(
    pool: &PgPool,
    guild_id: i64,
    user_id: i64,
) -> anyhow::Result<Option<UserPerm>> {
    let rec = match sqlx::query!(
        r#"
        SELECT perm_level
        FROM perms
        WHERE guild_id = $1 AND user_id = $2"#,
        guild_id,
        user_id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(row) => row,
        None => return Ok(None),
    };

    Ok(Some(rec.perm_level.into()))
}

pub async fn set_user_perms(
    pool: &PgPool,
    guild_id: i64,
    user_id: i64,
    perm_level: UserPerm,
) -> anyhow::Result<UserPerm> {
    let perm_level: i16 = perm_level.into();

    let rec = sqlx::query!(
        r#"
        INSERT INTO perms (guild_id, user_id, perm_level) VALUES ($1, $2, $3)
        ON CONFLICT (guild_id, user_id)
        DO UPDATE SET perm_level = EXCLUDED.perm_level
        RETURNING perm_level
        "#,
        guild_id,
        user_id,
        perm_level
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.perm_level.into())
}

#[derive(Debug)]
pub struct UserIdPermLevel {
    pub user_id: i64,
    pub perm_level: i16,
}

pub async fn get_all_users_with_perm(
    pool: &PgPool,
    guild_id: i64,
    perm_level: UserPerm,
) -> anyhow::Result<Vec<UserIdPermLevel>> {
    let perm_level: i16 = perm_level.into();

    let rec: Vec<UserIdPermLevel> = sqlx::query_as!(
        UserIdPermLevel,
        r#"
        SELECT user_id, perm_level
        FROM perms
        WHERE guild_id = $1 AND perm_level = $2
        "#,
        guild_id,
        perm_level
    )
    .fetch_all(pool)
    .await?;

    Ok(rec)
}

#[derive(Debug)]
pub struct GuildIdUserId {
    pub guild_id: i64,
    pub user_id: i64,
}

pub async fn delete_user(
    pool: &PgPool,
    guild_id: i64,
    user_id: i64,
) -> anyhow::Result<Option<GuildIdUserId>> {
    let rec: GuildIdUserId = match sqlx::query_as!(
        GuildIdUserId,
        r#"
        DELETE FROM perms
        WHERE user_id = $1 AND guild_id = $2
        RETURNING guild_id, user_id"#,
        user_id,
        guild_id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(row) => row,
        None => return Ok(None),
    };

    Ok(Some(rec))
}

pub async fn delete_guild(pool: &PgPool, guild_id: i64) -> anyhow::Result<Option<i64>> {
    let rec = match sqlx::query!(
        r#"
        DELETE FROM guilds
        WHERE guild_id = $1
        RETURNING guild_id"#,
        guild_id
    )
    .fetch_optional(pool)
    .await?
    {
        Some(row) => row,
        None => return Ok(None),
    };

    Ok(Some(rec.guild_id))
}

pub async fn insert_guild(pool: &PgPool, guild_id: i64) -> anyhow::Result<PgQueryResult> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO guilds
        VALUES ($1)
        ON CONFLICT DO NOTHING"#,
        guild_id
    )
    .execute(pool)
    .await?;

    Ok(rec)
}

#[derive(Debug)]
pub struct GuildIdPrefix {
    pub guild_id: i64,
    pub prefix: String,
}

pub async fn set_guild_prefix(
    pool: &PgPool,
    prefix_cache: PrefixCacheInternal,
    guild_id: i64,
    prefix: &str,
) -> anyhow::Result<GuildIdPrefix> {
    prefix_cache.insert(GuildId(guild_id.try_into().unwrap()), prefix.to_string());

    let rec = sqlx::query_as!(
        GuildIdPrefix,
        r#"
        INSERT INTO prefixes (guild_id, prefix)
        VALUES ($1, $2)
        ON CONFLICT (guild_id)
        DO UPDATE SET prefix = EXCLUDED.prefix
        RETURNING guild_id, prefix"#,
        guild_id,
        prefix
    )
    .fetch_one(pool)
    .await?;

    warn!("{:?}", prefix_cache);

    Ok(rec)
}

pub async fn get_guild_prefix(
    pool: &PgPool,
    prefix_cache: PrefixCacheInternal,
    guild_id: i64,
) -> anyhow::Result<String> {
    if let Some(prefix) = prefix_cache.get(&GuildId(guild_id.try_into().unwrap())) {
        warn!("{:?}", prefix_cache);
        Ok(prefix.clone())
    } else {
        let prefix = sqlx::query!(
            r#"
        SELECT prefix
        FROM prefixes
        WHERE guild_id = $1"#,
            guild_id
        )
        .fetch_optional(pool)
        .await?
        .map(|rec| rec.prefix)
        .unwrap_or_else(|| "~".to_string());

        prefix_cache.insert(GuildId(guild_id.try_into().unwrap()), prefix.clone());

        warn!("{:?}", prefix_cache);

        Ok(prefix)
    }
}

pub async fn delete_guild_prefix(
    pool: &PgPool,
    prefix_cache: PrefixCacheInternal,
    guild_id: i64,
) -> anyhow::Result<()> {
    prefix_cache.insert(GuildId(guild_id.try_into().unwrap()), "~".to_string());

    sqlx::query!(
        r#"
        DELETE FROM prefixes
        WHERE guild_id = $1"#,
        guild_id
    )
    .execute(pool)
    .await?;

    warn!("{:?}", prefix_cache);

    Ok(())
}
