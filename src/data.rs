use serenity::{client::bridge::gateway::ShardManager, model::prelude::*, prelude::*};
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct SongAuthorContainer;

impl TypeMapKey for SongAuthorContainer {
    type Value = Arc<RwLock<HashMap<uuid::Uuid, UserId>>>;
}

pub struct PoolContainer;

impl TypeMapKey for PoolContainer {
    type Value = PgPool;
}

pub struct ReqwestClientContainer;

impl TypeMapKey for ReqwestClientContainer {
    type Value = reqwest::Client;
}
