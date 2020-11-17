use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    model::prelude::Activity,
    model::{event::ResumedEvent, id::GuildId, prelude::Ready},
    prelude::*,
};
use std::sync::Arc;
use tracing::info;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        ctx.set_activity(Activity::playing(&format!("with {} guilds", guilds.len())))
            .await;
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}
