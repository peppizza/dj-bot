use serenity::{async_trait, model::prelude::*, prelude::*};

use tracing::{error, info};

use crate::{
    data::{DjOnlyContainer, PoolContainer},
    db::{delete_guild, delete_user, insert_guild},
    dj_only_store::delete_guild_from_store,
    queue::QueueMap,
};

pub struct Handler;

impl Handler {
    pub fn new() -> Self {
        Self {}
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
            let con = data.get::<DjOnlyContainer>().unwrap().clone();

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

            if let Err(e) = delete_guild_from_store(con, incomplete.id).await {
                error!("Error removing dj_only from deleted guild: {:?}", e);
            }
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if is_new {
            let guild_id = guild.id;
            info!("Added to guild: {}", guild_id);
            let data = ctx.data.read().await;
            let pool = data.get::<PoolContainer>().unwrap();

            match insert_guild(pool, guild_id.into()).await {
                Ok(guild_id) => {
                    info!("Added guild: {:?}", guild_id);
                }
                Err(why) => error!("Could not enter guild: {:?}", why),
            }
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

    async fn voice_state_update(
        &self,
        ctx: Context,
        _: Option<GuildId>,
        old: Option<VoiceState>,
        new: VoiceState,
    ) {
        if new.user_id != ctx.cache.current_user_id().await {
            return;
        }

        let guild_id = new.guild_id.unwrap();

        if new.channel_id.is_none() {
            let manager = songbird::get(&ctx).await.unwrap();

            if let Some(old) = old {
                if old.channel_id.is_some() {
                    if let Some(handler_lock) = manager.get(guild_id) {
                        {
                            let data = ctx.data.read().await;
                            let queue_container = data.get::<QueueMap>().unwrap().clone();
                            let queue = queue_container.remove(&guild_id).unwrap();
                            queue.1.stop();
                            let mut handler = handler_lock.lock().await;
                            handler.remove_all_global_events();
                        }
                        let _ = manager.remove(guild_id).await;
                    }
                }
            } else if let Some(handler_lock) = manager.get(guild_id) {
                {
                    let data = ctx.data.read().await;
                    let queue_container = data.get::<QueueMap>().unwrap().clone();
                    let queue = queue_container.remove(&guild_id).unwrap();
                    queue.1.stop();
                    let mut handler = handler_lock.lock().await;
                    handler.remove_all_global_events();
                }
                let _ = manager.remove(guild_id).await;
            }
        }
    }
}
