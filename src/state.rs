use serenity::{async_trait, client::bridge::gateway::ShardManager, model::prelude::*, prelude::*};
use songbird::input::Metadata;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use tracing::{error, info};

use crate::db::delete_guild;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    async fn guild_delete(&self, ctx: Context, incomplete: GuildUnavailable, _: Option<Guild>) {
        if !incomplete.unavailable {
            info!("Removed from guild: {}", incomplete.id);
            let data = ctx.data.read().await;
            let pool = data.get::<PoolContainer>().unwrap();

            match delete_guild(pool, incomplete.id.into()).await {
                Ok(guild_id) => {
                    if let Some(guild_id) = guild_id {
                        info!("Removed db entries for {}", guild_id);
                    } else {
                        info!("There was no entries for {}", incomplete.id);
                    }
                }
                Err(why) => error!("Could not remove db entries: {:?}", why),
            };
        }
    }
}

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct SongMetadataContainer;

pub struct MetadataWithAuthor {
    pub metadata: Arc<Metadata>,
    pub author: UserId,
}

impl TypeMapKey for SongMetadataContainer {
    type Value = Arc<RwLock<HashMap<uuid::Uuid, MetadataWithAuthor>>>;
}

pub struct PoolContainer;

impl TypeMapKey for PoolContainer {
    type Value = PgPool;
}
