use redis::{aio::MultiplexedConnection, RedisResult};
use serenity::model::id::GuildId;

pub async fn check_if_guild_in_store(
    con: &MultiplexedConnection,
    guild_id: GuildId,
) -> RedisResult<bool> {
    let mut con = con.clone();
    let is_in_store: bool = redis::cmd("EXISTS")
        .arg(guild_id.to_string())
        .query_async(&mut con)
        .await?;

    Ok(is_in_store)
}

pub async fn insert_guild_into_store(
    con: &MultiplexedConnection,
    guild_id: GuildId,
) -> RedisResult<()> {
    let mut con = con.clone();
    redis::cmd("SET")
        .arg(guild_id.to_string())
        .arg(0)
        .query_async(&mut con)
        .await?;

    Ok(())
}

pub async fn delete_guild_from_store(
    con: &MultiplexedConnection,
    guild_id: GuildId,
) -> RedisResult<()> {
    let mut con = con.clone();
    redis::cmd("DEL")
        .arg(guild_id.to_string())
        .query_async(&mut con)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use serenity::model::id::GuildId;

    use super::{check_if_guild_in_store, delete_guild_from_store, insert_guild_into_store};

    #[tokio::test]
    async fn test_key_exists() {
        dotenv::dotenv().ok();
        let client = redis::Client::open(std::env::var("REDIS_URL").unwrap()).unwrap();

        let con = client.get_multiplexed_tokio_connection().await.unwrap();

        let result = check_if_guild_in_store(&con, GuildId(123456789)).await;

        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_guild_inserting() {
        dotenv::dotenv().ok();
        let client = redis::Client::open(std::env::var("REDIS_URL").unwrap()).unwrap();
        let con = client.get_multiplexed_tokio_connection().await.unwrap();
        insert_guild_into_store(&con, GuildId(123456789))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_guild_deletion() {
        dotenv::dotenv().ok();
        let client = redis::Client::open(std::env::var("REDIS_URL").unwrap()).unwrap();
        let con = client.get_multiplexed_tokio_connection().await.unwrap();
        delete_guild_from_store(&con, GuildId(123456789))
            .await
            .unwrap();
    }
}
