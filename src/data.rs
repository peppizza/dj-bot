use bb8_redis::{bb8::Pool, RedisConnectionManager};
use dashmap::DashMap;
use serenity::{client::bridge::gateway::ShardManager, model::id::GuildId, prelude::*};
use sqlx::PgPool;
use std::sync::Arc;

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct PoolContainer;

impl TypeMapKey for PoolContainer {
    type Value = PgPool;
}

pub struct ReqwestClientContainer;

impl TypeMapKey for ReqwestClientContainer {
    type Value = reqwest::Client;
}

pub struct DjOnlyContainer;

pub type RedisPool = Pool<RedisConnectionManager>;

impl TypeMapKey for DjOnlyContainer {
    type Value = RedisPool;
}

pub struct PrefixCache;

pub type PrefixCacheInternal = Arc<DashMap<GuildId, String>>;

impl TypeMapKey for PrefixCache {
    type Value = PrefixCacheInternal;
}
