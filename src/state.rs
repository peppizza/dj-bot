use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    http::Http,
    model::prelude::Activity,
    model::{
        event::ResumedEvent,
        id::{ChannelId, GuildId},
        prelude::Ready,
    },
    prelude::*,
};
use songbird::{tracks::TrackQueue, Event, EventContext, EventHandler as VoiceEventHandler};
use std::{collections::HashMap, sync::Arc};
use tracing::{error, info};

pub struct VoiceQueueManager;

impl TypeMapKey for VoiceQueueManager {
    type Value = Arc<Mutex<HashMap<GuildId, TrackQueue>>>;
}

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

pub struct TrackEndNotifier {
    pub chan_id: ChannelId,
    pub http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            if let Err(why) = self
                .chan_id
                .say(&self.http, format!("Tracks ended: {}", track_list.len()))
                .await
            {
                error!("{}", why);
            }
        }

        None
    }
}
