use serenity::{async_trait, client::Cache, http::Http, model::prelude::*, prelude::*};
use songbird::{Call, Event, EventContext, EventHandler as VoiceEventHandler};

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::util::AuthorContainerLock;

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

pub struct RemoveFromAuthorMap {
    pub map: AuthorContainerLock,
}

#[async_trait]
impl VoiceEventHandler for RemoveFromAuthorMap {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(&[(_, handle)]) = ctx {
            let uuid = handle.uuid();
            let mut map = self.map.write().await;
            map.remove(&uuid);
        }

        None
    }
}

pub struct ChannelIdleChecker {
    pub handler_lock: Arc<Mutex<Call>>,
    pub elapsed: Arc<AtomicUsize>,
    pub chan_id: ChannelId,
    pub guild_id: GuildId,
    pub http: Arc<Http>,
    pub cache: Arc<Cache>,
}

#[async_trait]
impl VoiceEventHandler for ChannelIdleChecker {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let mut handler = self.handler_lock.lock().await;

        if handler.queue().is_empty() {
            if (self.elapsed.fetch_add(1, Ordering::Relaxed) + 1) > 5 {
                let guild = match self.cache.guild(self.guild_id).await {
                    Some(guild) => guild,
                    None => {
                        let _ = handler.leave().await;
                        return Some(Event::Cancel);
                    }
                };

                if guild
                    .voice_states
                    .get(&self.cache.current_user_id().await)
                    .is_some()
                {
                    let _ = handler.leave().await;
                    let _ = self
                        .chan_id
                        .say(&self.http, "I left the channel due to inactivity")
                        .await;
                }

                return Some(Event::Cancel);
            }
        } else {
            self.elapsed.store(0, Ordering::Relaxed);
        }

        None
    }
}
