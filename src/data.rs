use flume::Sender;
use redis::aio::MultiplexedConnection;
use serenity::{client::bridge::gateway::ShardManager, model::id::GuildId, prelude::*};
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};

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

impl TypeMapKey for DjOnlyContainer {
    type Value = MultiplexedConnection;
}

pub struct StopContainer;

impl TypeMapKey for StopContainer {
    type Value = Arc<Mutex<HashMap<GuildId, Sender<()>>>>;
}
