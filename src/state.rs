use serenity::{
    async_trait, client::bridge::gateway::ShardManager, http::Http, model::prelude::*, prelude::*,
};
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler};
use sqlx::PgPool;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tracing::{debug, error, info};

use crate::db::{delete_guild, delete_user};

lazy_static::lazy_static! {
    static ref DBL_API_KEY: String = std::env::var("DBL_API_KEY").expect("Expected DBL_API_KEY in dotenv file");
}

pub struct Handler {
    is_loop_running: AtomicBool,
}

impl Handler {
    pub fn new() -> Self {
        Self {
            is_loop_running: AtomicBool::new(false),
        }
    }
}

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

    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        _: Option<Member>,
    ) {
        let data = ctx.data.read().await;
        let pool = data.get::<PoolContainer>().unwrap();
        match delete_user(pool, guild_id.into(), user.id.into()).await {
            Ok(guild_and_user_id) => {
                if let Some(guild_and_user_id) = guild_and_user_id {
                    info!("Removed db entry: {:?}", guild_and_user_id)
                } else {
                    info!(
                        "There was no entry for user: {}, guild: {}",
                        user.id, guild_id
                    )
                }
            }
            Err(e) => error!("Could not remove db entries: {:?}", e),
        }
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        debug!("Cache built");

        let ctx = Arc::new(ctx);

        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);

            tokio::spawn(async move {
                let ctx = ctx1.clone();

                loop {
                    debug!("running empty channel loop");

                    let guilds = ctx.cache.guilds().await;

                    for guild_id in guilds {
                        let guild = match guild_id.to_guild_cached(&ctx).await {
                            Some(guild) => guild,
                            None => continue,
                        };

                        let bot_voice_channel = match guild
                            .voice_states
                            .get(&ctx.cache.current_user_id().await)
                            .map(|state| state.channel_id)
                        {
                            Some(c) => c,
                            None => continue,
                        };

                        let bot_voice_channel = match bot_voice_channel {
                            Some(c) => c,
                            None => continue,
                        };

                        let count_of_members = guild
                            .voice_states
                            .values()
                            .filter(|state| match state.channel_id {
                                Some(c) => c == bot_voice_channel,
                                None => false,
                            })
                            .count();

                        debug!("{}", count_of_members);

                        if count_of_members == 1 {
                            let manager = songbird::get(&ctx).await.unwrap();

                            if let Some(handler_lock) = manager.get(guild_id) {
                                let handler = handler_lock.lock().await;
                                let queue = handler.queue();
                                let current_queue = queue.current_queue();

                                if !current_queue.is_empty() {
                                    let data = ctx.data.read().await;
                                    let author_container_lock =
                                        data.get::<SongAuthorContainer>().unwrap().clone();
                                    let mut author_container = author_container_lock.write().await;

                                    for track in current_queue {
                                        author_container.remove(&track.uuid());
                                    }
                                }
                            }

                            let _ = manager.remove(guild_id).await;
                        }
                    }

                    tokio::time::delay_for(Duration::from_secs(5 * 60)).await;
                }
            });

            let ctx2 = Arc::clone(&ctx);

            tokio::spawn(async move {
                let ctx = Arc::clone(&ctx2);

                loop {
                    debug!("Running presence update loop");

                    let server_count = ctx.cache.guild_count().await;

                    ctx.set_activity(Activity::listening(&format!("{} servers", server_count)))
                        .await;

                    let data = ctx.data.read().await;
                    let client = data.get::<ReqwestClientContainer>().unwrap().clone();

                    let shard_id = ctx.shard_id;
                    let shard_count = ctx.cache.shard_count().await;

                    let _ = client
                        .post("https://top.gg/api/bots/stats")
                        .json(&serde_json::json!({
                            "server_count": server_count,
                            "shard_id": shard_id,
                            "shard_count": shard_count,
                        }))
                        .bearer_auth(DBL_API_KEY.clone())
                        .send()
                        .await;

                    tokio::time::delay_for(Duration::from_secs(30 * 60)).await;
                }
            });

            self.is_loop_running.store(true, Ordering::Relaxed);
        }
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        guild_id: Option<GuildId>,
        old: Option<VoiceState>,
        new: VoiceState,
    ) {
        if new.user_id != ctx.cache.current_user_id().await {
            return;
        }

        let guild_id = guild_id.unwrap();

        if new.channel_id.is_none() {
            let manager = songbird::get(&ctx).await.unwrap();

            if let Some(old) = old {
                if old.channel_id.is_some() {
                    if let Some(handler_lock) = manager.get(guild_id) {
                        let handler = handler_lock.lock().await;
                        let queue = handler.queue();
                        let current_queue = queue.current_queue();

                        if !current_queue.is_empty() {
                            let data = ctx.data.read().await;
                            let author_container_lock =
                                data.get::<SongAuthorContainer>().unwrap().clone();
                            let mut author_container = author_container_lock.write().await;

                            for track in current_queue {
                                author_container.remove(&track.uuid());
                            }
                        }
                    }

                    let _ = manager.remove(new.guild_id.unwrap()).await;
                }
            } else {
                if let Some(handler_lock) = manager.get(guild_id) {
                    let handler = handler_lock.lock().await;
                    let queue = handler.queue();
                    let current_queue = queue.current_queue();

                    if !current_queue.is_empty() {
                        let data = ctx.data.read().await;
                        let author_container_lock =
                            data.get::<SongAuthorContainer>().unwrap().clone();
                        let mut author_container = author_container_lock.write().await;

                        for track in current_queue {
                            author_container.remove(&track.uuid());
                        }
                    }
                }

                let _ = manager.remove(new.guild_id.unwrap()).await;
            }
        }
    }
}

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

pub struct TrackStartNotifier {
    pub chan_id: ChannelId,
    pub http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(&[(_, handle)]) = ctx {
            let metadata = handle.metadata();
            let title = metadata.title.clone().unwrap_or_default();
            let url = metadata.source_url.clone().unwrap_or_default();
            let _ = self
                .chan_id
                .send_message(&self.http, |m| {
                    m.embed(|e| {
                        e.title("Now playing");
                        e.description(format!("[{}]({})", title, url));

                        e
                    })
                })
                .await;
        }

        None
    }
}
