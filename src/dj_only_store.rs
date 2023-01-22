use anyhow::Result;
use bb8_redis::redis;
use serenity::model::id::GuildId;

use crate::data::RedisPool;

pub async fn check_if_guild_in_store(pool: RedisPool, guild_id: GuildId) -> Result<bool> {
    let mut con = pool.get().await?;

    let is_in_store: bool = redis::cmd("EXISTS")
        .arg(guild_id.to_string())
        .query_async(&mut *con)
        .await?;

    Ok(is_in_store)
}

pub async fn insert_guild_into_store(pool: RedisPool, guild_id: GuildId) -> Result<()> {
    let mut con = pool.get().await?;

    redis::cmd("SET")
        .arg(guild_id.to_string())
        .arg(0)
        .query_async(&mut *con)
        .await?;

    Ok(())
}

pub async fn delete_guild_from_store(pool: RedisPool, guild_id: GuildId) -> Result<()> {
    let mut con = pool.get().await?;

    redis::cmd("DEL")
        .arg(guild_id.to_string())
        .query_async(&mut *con)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use bb8_redis::{bb8, RedisConnectionManager};
    use serenity::model::id::GuildId;

    use super::{check_if_guild_in_store, delete_guild_from_store, insert_guild_into_store};

    #[tokio::test]
    async fn test_key_exists() {
        dotenv::dotenv().ok();
        let manager = RedisConnectionManager::new(std::env::var("REDIS_URL").unwrap()).unwrap();
        let pool = bb8::Pool::builder().build(manager).await.unwrap();

        let result = check_if_guild_in_store(pool, GuildId(123456789)).await;

        println!("{result:?}");
    }

    #[tokio::test]
    async fn test_guild_inserting() {
        dotenv::dotenv().ok();
        let manager = RedisConnectionManager::new(std::env::var("REDIS_URL").unwrap()).unwrap();
        let pool = bb8::Pool::builder().build(manager).await.unwrap();
        insert_guild_into_store(pool, GuildId(123456789))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_guild_deletion() {
        dotenv::dotenv().ok();
        let manager = RedisConnectionManager::new(std::env::var("REDIS_URL").unwrap()).unwrap();
        let pool = bb8::Pool::builder().build(manager).await.unwrap();
        delete_guild_from_store(pool, GuildId(123456789))
            .await
            .unwrap();
    }
}
