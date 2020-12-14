use serenity::{client::bridge::gateway::ShardManager, prelude::*};
use sqlx::PgPool;
use std::sync::Arc;

use crate::util::AuthorContainerLock;

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct SongAuthorContainer;

impl TypeMapKey for SongAuthorContainer {
    type Value = AuthorContainerLock;
}

pub struct PoolContainer;

impl TypeMapKey for PoolContainer {
    type Value = PgPool;
}

pub struct ReqwestClientContainer;

impl TypeMapKey for ReqwestClientContainer {
    type Value = reqwest::Client;
}
