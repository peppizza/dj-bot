use serenity::{async_trait, client::Cache, http::Http, model::prelude::*, prelude::*};
use songbird::{Call, Event, EventContext, EventHandler as VoiceEventHandler};

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::queue::Queue;

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

pub struct ChannelIdleChecker {
    pub handler_lock: Arc<Mutex<Call>>,
    pub elapsed: AtomicUsize,
    pub chan_id: ChannelId,
    pub guild_id: GuildId,
    pub http: Arc<Http>,
    pub cache: Arc<Cache>,
    pub queue: Queue,
}

#[async_trait]
impl VoiceEventHandler for ChannelIdleChecker {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let mut handler = self.handler_lock.lock().await;

        if self.queue.is_empty() {
            if (self.elapsed.fetch_add(1, Ordering::Relaxed) + 1) > 5 {
                let _ = handler.leave().await;
                let _ = self
                    .chan_id
                    .say(&self.http, "I left the channel due to inactivity")
                    .await;

                return Some(Event::Cancel);
            }
        } else {
            self.elapsed.store(0, Ordering::Relaxed);
        }

        None
    }
}
