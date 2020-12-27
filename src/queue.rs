use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use serenity::{
    async_trait,
    client::Context,
    model::id::GuildId,
    prelude::{RwLock, TypeMapKey},
};
use songbird::{
    create_player,
    input::{Input, Restartable},
    tracks::{TrackHandle, TrackResult},
    Call, Event, EventContext, EventHandler, TrackEvent,
};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct QueuedTrack {
    pub name: String,
    pub uuid: Uuid,
    pub length: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct Queue {
    inner: Arc<Mutex<QueueCore>>,
}

#[derive(Debug, Default)]
pub struct QueueCore {
    tracks: VecDeque<QueuedTrack>,
    current_track: Option<TrackHandle>,
}

struct PlayNextTrack {
    remote_lock: Arc<Mutex<QueueCore>>,
    driver: Arc<Mutex<Call>>,
    should_not_check_uuid: AtomicBool,
}

#[async_trait]
impl EventHandler for PlayNextTrack {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let track = {
            let mut inner = self.remote_lock.lock();

            if !self.should_not_check_uuid.load(Ordering::Relaxed) {
                let front_ended = match ctx {
                    EventContext::Track(ts) => {
                        let queue_uuid = inner.tracks.front().map(|input| input.uuid);
                        let ended_uuid = ts.first().map(|handle| handle.1.uuid());

                        queue_uuid.is_some() && queue_uuid == ended_uuid
                    }
                    _ => false,
                };

                if !front_ended {
                    return None;
                }
            }

            inner.tracks.pop_front();

            info!("Queued track ended: {:?}.", ctx);
            info!("{} tracks remain.", inner.tracks.len());

            if let Some(track) = inner.tracks.front() {
                Some(track.clone())
            } else {
                None
            }
        };

        if let Some(track) = track {
        } else {
            self.should_not_check_uuid.store(true, Ordering::Relaxed);
        }

        None
    }
}

async fn get_input_from_queued_track(track: QueuedTrack) -> Result<Input> {
    match Restartable::ytdl_search(&track.name).await {
        Ok(r) => Ok(Input::from(r)),
        Err(e) => Err(anyhow!("{:?}", e)),
    }
}

impl Queue {
    pub async fn add(&self, input: QueuedTrack, driver: Arc<Mutex<Call>>) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        let url = input.url.clone();
        inner.tracks.push_front(input);
        if self.is_empty().await {
            let input = match Restartable::ytdl(url).await {
                Ok(input) => input,
                Err(e) => return Err(anyhow!("{:?}", e)),
            };
            let (track, handle) = create_player(Input::from(input));
            handle.add_event(
                Event::Track(TrackEvent::End),
                PlayNextTrack {
                    driver: driver.clone(),
                    remote_lock: self.inner.clone(),
                },
            );
            let mut handler = driver.lock().await;
            handler.play(track);
            inner.current_track = Some(handle);
        }
        Ok(())
    }

    pub async fn is_empty(&self) -> bool {
        let inner = self.inner.lock().await;

        inner.tracks.is_empty()
    }

    pub async fn len(&self) -> usize {
        let inner = self.inner.lock().await;

        inner.tracks.len()
    }

    pub async fn pause(&self) -> TrackResult<()> {
        let inner = self.inner.lock().await;

        if let Some(handle) = &inner.current_track {
            handle.pause()
        } else {
            Ok(())
        }
    }

    pub async fn resume(&self) -> TrackResult<()> {
        let inner = self.inner.lock().await;

        if let Some(handle) = &inner.current_track {
            handle.play()
        } else {
            Ok(())
        }
    }

    pub async fn stop(&self) {
        let mut inner = self.inner.lock().await;

        if let Some(handle) = &inner.current_track {
            let _ = handle.stop();
        }

        inner.tracks.clear();
    }

    pub async fn skip(&mut self) -> anyhow::Result<()> {
        let inner = self.inner.lock().await;

        if let Some(handle) = &inner.current_track {
            handle.stop()?;
        }
        Ok(())
    }

    pub async fn current(&self) -> Option<TrackHandle> {
        let inner = self.inner.lock().await;

        inner.current_track.clone()
    }

    pub async fn current_queue(&self) -> Vec<QueuedTrack> {
        let inner = self.inner.lock().await;

        inner.tracks.iter().map(|input| input.clone()).collect()
    }

    pub async fn dequeue(&self, index: usize) -> Option<QueuedTrack> {
        self.modify_queue(|vq| vq.remove(index)).await
    }

    pub async fn modify_queue<F, O>(&self, func: F) -> O
    where
        F: FnOnce(&mut VecDeque<QueuedTrack>) -> O,
    {
        let mut inner = self.inner.lock().await;

        func(&mut inner.tracks)
    }
}

pub struct QueueMap;

impl TypeMapKey for QueueMap {
    type Value = Arc<RwLock<HashMap<GuildId, Queue>>>;
}

pub async fn get_queue_from_ctx_and_guild_id(ctx: &Context, guild_id: GuildId) -> Queue {
    let data = ctx.data.read().await;
    let queue_container_lock = data.get::<QueueMap>().unwrap().clone();
    let queue_container = queue_container_lock.read().await;
    let queue = queue_container.get(&guild_id).unwrap().clone();

    queue
}
